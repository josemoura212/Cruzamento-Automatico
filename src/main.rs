use std::thread::sleep;
use std::time::Duration;

mod comunicacao;
mod controlador;
mod transito;

use transito::Transito;
use transito::Via;

use controlador::Controlador;

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

// Simula transito até os carros saírem do perímetro ou colidirem
fn simula_mundo() {
    const TICKMS: f64 = 100.0; // Passo da simulação, em ms
    const TEMPO_ENTRE_CHEGADAS: f64 = 3000.0; // Tempo entre chegadas de carros, em ms
    const TEMPO_ENTRE_CONTROLES: f64 = 1000.0; // Tempo entre ações de controle, em ms

    // Cria um sistema de comunicação
    let mut comunicacao = Comunicacao::new();

    // Cria uma descrição de trânsito
    let mut transito = Transito::new();

    // Cria o primeiro carro da via H	!!!
    if let Err(msg) = transito.chega_carro(Via::ViaH, &mut comunicacao) {
        println!("Falha em chegar o primeiro carro na via H: {}", msg);
    }

    // Cria o primeiro carro da via V	!!!
    if let Err(msg) = transito.chega_carro(Via::ViaV, &mut comunicacao) {
        println!("Falha em chegar o primeiro carro na via V: {}", msg);
    }

    // Tempo até a próxima chegada de um carro
    let mut tempo_ateh_proxima_chegada = TEMPO_ENTRE_CHEGADAS;

    // Cria uma descrição de controlador
    let mut controlador = Controlador::new();

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
        match transito.ocorreu_colisao() {
            //Some(m) => panic!( "Ocorreu colisao: {}", m),		!!!
            Some(m) => {
                println!("Ocorreu colisao: {}", m);
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
            //assert!( transito.chega_carro( Via::ViaH, &mut comunicacao), "Falha em chegar um carro via H"); !!!
            //assert!( transito.chega_carro( Via::ViaV, &mut comunicacao), "Falha em chegar um carro via V"); !!!

            match transito.chega_carro(Via::ViaH, &mut comunicacao) {
                Ok(_) => (),
                Err(msg) => println!("Falha em chegar um carro via H: {}", msg),
            }

            match transito.chega_carro(Via::ViaV, &mut comunicacao) {
                Ok(_) => (),
                Err(msg) => println!("Falha em chegar um carro via V: {}", msg),
            }

            tempo_ateh_proxima_chegada += TEMPO_ENTRE_CHEGADAS;
        }

        // Verifica se está na hora de chamar o controlador
        tempo_ateh_proximo_controle -= TICKMS;

        if tempo_ateh_proximo_controle <= 0.0 {
            controlador.controle(TEMPO_ENTRE_CONTROLES, &mut comunicacao);
            tempo_ateh_proximo_controle += TEMPO_ENTRE_CONTROLES;
        }
    }
}

fn main() {
    println!("Inicio da simulação de cruzamento automático");
    simula_mundo();
    println!("Fim da simulação de cruzamento automático");
}
