use std::thread::sleep;
use std::time::Duration;

use rand::Rng; // Este não é da biblioteca padrão, requer [dependencies] rand = "0.8.5"

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

const TEMPO_MIN_ENTRE_CHEGADAS: f64 = 3000.0; // Tempo mínimo entre chegadas de carros, em ms
const TEMPO_MAX_ENTRE_CHEGADAS: f64 = 5000.0; // Tempo máximo entre chegadas de carros, em ms

// Sorteia tempo até a chegada do próximo veículo, distribuição uniforme
fn tempo_entre_chegadas() -> f64 {
    rand::thread_rng().gen_range(TEMPO_MIN_ENTRE_CHEGADAS..=TEMPO_MAX_ENTRE_CHEGADAS)
}

// Simula transito até os carros saírem do perímetro ou colidirem
fn simula_mundo() {
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
    let mut tempo_ateh_proxima_chegada = tempo_entre_chegadas();

    // Cria uma estrutura de controle
    let mut controle = Controle::new(TipoControlador::Semaforo);
    //	let mut controle = Controle::new(TipoControlador::FAZNADA);

    // Tempo até a próxima ação de controle
    let mut tempo_ateh_proximo_controle = TEMPO_ENTRE_CONTROLES;

    println!("simula_transito");

    loop {
        // Mantém o tempo simulado próximo do tempo real, dorme TICKMS ms
        sleep(Duration::from_millis(TICKMS.round() as u64));

        // Atualiza estado do trânsito
        transito.tick(TICKMS, &mut comunicacao);

        // Mostra estado das vias
        transito.mostra_vias();

        // Aborta a simulação se ocorreu colisão
        if let Some(m) = transito.ocorreu_colisao() {
            println!("Ocorreu colisao: {}", m);
            return;
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

            tempo_ateh_proxima_chegada += tempo_entre_chegadas(); // !!!
        }

        // Verifica se está na hora de chamar o controlador
        tempo_ateh_proximo_controle -= TICKMS;

        if tempo_ateh_proximo_controle <= 0.0 {
            controle.acao_controle(TEMPO_ENTRE_CONTROLES, &mut comunicacao);
            tempo_ateh_proximo_controle += TEMPO_ENTRE_CONTROLES;
        }
    }
}

fn main() {
    println!("Inicio da simulação de cruzamento automático");
    simula_mundo();
    println!("Fim da simulação de cruzamento automático");
}
