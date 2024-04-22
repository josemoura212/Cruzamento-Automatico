use std::thread::sleep;
use std::time::Duration;

mod mundo;

use mundo::Transito;
use mundo::Via;

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

// Simula carros até saírem do perímetro ou colidirem
// Retorna se houve colisão ou não
fn simula_carros() {
    const TEMPO_ENTRE_CHEGADAS: f64 = 3000.0; // Tempo entre chegadas em ms

    // Cria uma descrição de trânsito
    let mut transito = Transito::new();

    // Cria o primeiro carro da via H
    transito.chega_carro(Via::ViaH);

    // Cria o primeiro carro da via V
    transito.chega_carro(Via::ViaV);

    // Tempo até a próxima chegada de um carro
    let mut tempo_ateh_proxima_chegada = TEMPO_ENTRE_CHEGADAS;

    println!("simula_carros");
    let mut tickms: f64; // tempo que passou em cada tick, milisegundos

    loop {
        // Passos de 100ms
        tickms = 100.0;

        // Ao final do tick devemos atualizar o estado do carro
        sleep(Duration::from_millis(tickms.round() as u64));
        transito.tick(tickms);

        // Mostra estado das vias
        transito.mostra_vias();

        // Aborta a simulação se ocorreu colisão
        match transito.ocorreu_colisao() {
            Some(m) => panic!("Ocorreu colisao: {}", m),
            None => {}
        }

        // Verifica se algum carro no sistema
        if transito.vazio() {
            break;
        }

        // Verifica se está na hora de chegar novos carros
        tempo_ateh_proxima_chegada -= tickms;

        if tempo_ateh_proxima_chegada <= 0.0 {
            assert!(
                transito.chega_carro(Via::ViaH),
                "Falha em chegar um carro via H"
            );
            assert!(
                transito.chega_carro(Via::ViaV),
                "Falha em chegar um carro via V"
            );
            tempo_ateh_proxima_chegada += TEMPO_ENTRE_CHEGADAS;
        }
    }
}

fn main() {
    println!("Inicio da simulação de cruzamento automático");
    simula_carros();
    println!("Fim da simulação de cruzamento automático");
}
