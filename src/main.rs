use std::thread::sleep;
use std::time::Duration;

use rand::Rng; // Para gerar números aleatórios, não é 'std::'
               // Requer [dependencies] rand = "0.8.5"

use std::env; // Para acessar os argumentos da linha de comando, exemplo:
              // cargo run -- s|n  3  5

use device_query::{DeviceQuery, DeviceState, Keycode}; // Para acessar o teclado, não é 'std::'
                                                       // Requer [dependencies] device_query = "1.1.3"

mod comunicacao;
mod controlador;
mod transito;

use transito::Transito;
use transito::Via;

use controlador::{Controle, TipoControlador};

use comunicacao::Comunicacao;

/* Geometria do cruzamento
    Zero de cada via é o início do cruzamento

                        largura V
                margem V|    |
                        |    |
                        |    |margem H
------------------------+----+--------
    ViaH  > > > 	   	|    |	    largura H
------------------------+----+--------
    perímetro H			|    |
                        |    |
                        |    |
                        | ^  |perímetro V
                        | ^  |
                        | ^  |
                        |ViaV|
                        |    |


*/

// Sorteia tempo até a chegada do próximo veículo
fn tempo_entre_chegadas(min: f64, max: f64) -> f64 {
    rand::thread_rng().gen_range(min..=max)
}

// Simula transito até os carros saírem do perímetro ou colidirem
fn simula_mundo(cont: char, tec_min: f64, tec_max: f64) {
    const TICKMS: f64 = 100.0; // Passo da simulação, em ms
    const TEMPO_ENTRE_CONTROLES: f64 = 1000.0; // Tempo entre ações de controle, em ms

    // Cria um sistema de comunicação
    let mut comunicacao = Comunicacao::new();

    // Cria uma descrição de trânsito
    let mut transito = Transito::new();

    // Cria o primeiro carro da via H
    match transito.chega_carro(Via::ViaH, &mut comunicacao) {
        Ok(_) => (),
        Err(msg) => println!("Via H: {}", msg),
    };

    // Cria o primeiro carro da via V
    match transito.chega_carro(Via::ViaV, &mut comunicacao) {
        Ok(_) => (),
        Err(msg) => println!("Via V: {}", msg),
    };

    // Tempo até a próxima chegada de um carro
    let mut tempo_ateh_proxima_chegada = tempo_entre_chegadas(tec_min, tec_max);

    // Cria uma estrutura de controle
    let mut controle = match cont {
        's' => Controle::new(TipoControlador::SEMAFORO),
        'n' => Controle::new(TipoControlador::FAZNADA),
        _other => panic!("Tipo de controlador não é s|n."),
    };

    // Tempo até a próxima ação de controle
    let mut tempo_ateh_proximo_controle = TEMPO_ENTRE_CONTROLES;

    // Para leitura do teclado
    let device_state = DeviceState::new();

    println!("simula_transito");

    loop {
        // Mantém o tempo simulado próximo do tempo real, dorme TICKMS ms
        sleep(Duration::from_millis(TICKMS.round() as u64));

        // Atualiza estado do trânsito
        transito.tick(TICKMS, &mut comunicacao);

        // Mostra estado das vias
        transito.mostra_vias();

        // Aborta a simulação se ocorreu colisão
        match transito.ocorreu_colisao() {
            Some(m) => {
                println!(
                    "Ocorreu colisao, controlador {}, tempos entre {} e {}: {}",
                    cont, tec_min, tec_max, m
                );
                return;
            }
            None => {}
        }

        // Verifica se tem algum carro no sistema
        if transito.vazio() {
            println!("Nenhum carro no perímetro");
            break;
        }

        // Verifica se está na hora de chegar novos carros
        tempo_ateh_proxima_chegada -= TICKMS;

        if tempo_ateh_proxima_chegada <= 0.0 {
            match transito.chega_carro(Via::ViaH, &mut comunicacao) {
                Ok(_) => (),
                Err(msg) => println!("Falha em chegar um carro via H: {}", msg),
            }

            match transito.chega_carro(Via::ViaV, &mut comunicacao) {
                Ok(_) => (),
                Err(msg) => println!("Falha em chegar um carro via V: {}", msg),
            }

            tempo_ateh_proxima_chegada += tempo_entre_chegadas(tec_min, tec_max);
        }

        // Verifica se está na hora de chamar o controlador
        tempo_ateh_proximo_controle -= TICKMS;

        if tempo_ateh_proximo_controle <= 0.0 {
            controle.acao_controle(TEMPO_ENTRE_CONTROLES, &mut comunicacao);
            tempo_ateh_proximo_controle += TEMPO_ENTRE_CONTROLES;
        }

        // Verifica se algo foi teclado
        let keys: Vec<Keycode> = device_state.get_keys();
        if !keys.is_empty() && keys[0] == Keycode::P {
            // P Pausa
            loop {
                let keys2: Vec<Keycode> = device_state.get_keys();
                if !keys2.is_empty() && keys2[0] == Keycode::Backspace {
                    // Backspace libera
                    break;
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        panic!("Uso:  <s|n>  <min entre chegadas>  <max entre chegadas>");
    }

    let cont = args[1]
        .trim()
        .chars()
        .next()
        .expect("Uso: <s|n>  <min entre chegadas>  <max entre chegadas>");

    match cont {
        's' | 'n' => (),
        _other => panic!("Uso: <s|n>  <min entre chegadas>  <max entre chegadas>"),
    };

    let result_tec_min = args[2].trim().parse::<f64>();
    let tec_min = result_tec_min.expect("Uso: <s|n>  <min entre chegadas>  <max entre chegadas>");

    let result_tec_max = args[3].trim().parse::<f64>();
    let tec_max = result_tec_max.expect("Uso: <s|n>  <min entre chegadas>  <max entre chegadas>");

    if tec_min < 2.0 || tec_max < 2.0 {
        println!("Tempo entre chegadas deve ser no mínimo 2 segundos.");
        return;
    }

    if tec_min > tec_max {
        println!(
            "Tempo entre chegadas mínimo não pode ser maior que o tempo entre chegadas máximo."
        );
        return;
    }

    println!("Inicio da simulação de cruzamento automático");

    simula_mundo(cont, 1000.0 * tec_min, 1000.0 * tec_max);

    println!("Fim da simulação de cruzamento automático");
}

// cargo run       --     s  3  5
