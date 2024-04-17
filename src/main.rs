use std::thread::sleep;
use std::time::Duration;

/* Geometria do cruzamento

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

const _VIAH_MARGEM: f64 = 15.0; //metros
const _VIAV_MARGEM: f64 = 15.0; //metros

const VIAH_LARGURA: f64 = 4.0; //metros
const VIAV_LARGURA: f64 = 4.0; //metros

const _VIAH_PERIMETRO: f64 = 150.0; //metros
const _VIAV_PERIMETRO: f64 = 150.0; //metros

const _CARRO_LARGURA: f64 = 2.0; //metros
const CARRO_COMPRIMENTO: f64 = 4.0; //metros

// Velocidade máxima de qualquer veículo em metros por segundo
const VELOCIDADE_MAXIMA: f64 = 200.0 * (1000.0 / 3600.0);

// Aceleração máxima de qualquer veículo em metros por segundo ao quadrado
const ACELERACAO_MAXIMA: f64 = 3.0;

// Aceleração mínima de qualquer veículo em metros por segundo ao quadrado
const ACELERACAO_MINIMA: f64 = -10.0;

// Simula 2 carros até saírem do perímetro controlado ou colidirem
// Retorna se houve colisão ou não
fn simula_carros(via_carro1: char, acel_carro1: f64, via_carro2: char, acel_carro2: f64) -> bool {
    // Descrição do carro 1
    let chassi1: i32 = 1111; // identificação de um carro
    let via1: char = via_carro1; // via deste carro
    let _acel_max1 = ACELERACAO_MAXIMA; // metros por segundo ao quadrado
    let _acel_min1 = ACELERACAO_MINIMA; // metros por segundo ao quadrado
    let vel_max1 = VELOCIDADE_MAXIMA; // metros por segundo
    let comprimento1 = CARRO_COMPRIMENTO; // metros
    let mut pos_atual1: f64 = -80.0; // metros do cruzamento
    let mut vel_atual1: f64 = 0.0; // metros por segundo
    let acel_atual1: f64; // metros por segundo ao quadrado

    // Descrição do carro 2
    let chassi2: i32 = 2222; // identificação de um carro
    let via2: char = via_carro2; // via deste carro
    let _acel_max2 = ACELERACAO_MAXIMA; // metros por segundo ao quadrado
    let _acel_min2 = ACELERACAO_MINIMA; // metros por segundo ao quadrado
    let vel_max2 = VELOCIDADE_MAXIMA; // metros por segundo
    let comprimento2: f64 = CARRO_COMPRIMENTO; // metros
    let mut pos_atual2: f64 = -100.0; // metros do cruzamento
    let mut vel_atual2: f64 = 0.0; // metros por segundo
    let acel_atual2: f64; // metros por segundo ao quadrado

    acel_atual1 = acel_carro1;
    acel_atual2 = acel_carro2;

    println!("Início da simulação");
    let mut tickms: f64; // tempo que passou em cada tick, milisegundos

    loop {
        // Ao final do tick devemos atualizar o estado do carro
        sleep(Duration::from_millis(100));

        tickms = 100.0;

        // Atualiza o carro 1

        let old_position = pos_atual1;

        pos_atual1 = pos_atual1
            + vel_atual1 * (tickms / 1000.0)
            + acel_atual1 * (tickms / 1000.0) * (tickms / 1000.0) / 2.0;
        vel_atual1 = vel_atual1 + acel_atual1 * (tickms / 1000.0);

        // Restrições carro 1
        if pos_atual1 < old_position {
            // Não anda para tras
            pos_atual1 = old_position;
        }

        if vel_atual1 < 0.0 {
            // Não anda para tras
            vel_atual1 = 0.0;
        }

        if vel_atual1 > vel_max1 {
            vel_atual1 = vel_max1; // Trava na velocidade máxima
        }

        println!(
            "Carro1 {} na posição {}{}, velocidade {}, aceleração {}",
            chassi1, via1, pos_atual1, vel_atual1, acel_atual1
        );

        // Atualiza o carro 2

        let old_position = pos_atual2;

        pos_atual2 = pos_atual2
            + vel_atual2 * (tickms / 1000.0)
            + acel_atual2 * (tickms / 1000.0) * (tickms / 1000.0) / 2.0;
        vel_atual2 = vel_atual2 + acel_atual2 * (tickms / 1000.0);

        // Restrições carro 2
        if pos_atual2 < old_position {
            // Não anda para tras
            pos_atual2 = old_position;
        }

        if vel_atual2 < 0.0 {
            // Não anda para tras
            vel_atual2 = 0.0;
        }

        if vel_atual2 > vel_max2 {
            vel_atual2 = vel_max2; // Trava na velocidade máxima
        }

        println!(
            "Carro2 {} na posição {}{}, velocidade {}, aceleração {}",
            chassi2, via2, pos_atual2, vel_atual2, acel_atual2
        );

        // Detecta colisão na via H
        if via1 == 'H' && via2 == 'H' {
            if colisao_longitudinal(pos_atual1, comprimento1, pos_atual2) {
                println!("Colisão na via H");
                return true;
            }
        }

        // Detecta colisão na via V
        if via1 == 'V' && via2 == 'V' {
            if colisao_longitudinal(pos_atual1, comprimento1, pos_atual2) {
                println!("Colisão na via V");
                return true;
            }
        }

        // Detecta colisão no cruzamento
        if via1 != via2 {
            if dentro_cruzamento(pos_atual1, comprimento1, via1)
                && dentro_cruzamento(pos_atual2, comprimento2, via2)
            {
                println!("Colisão dentro do cruzamento");
                return true;
            }
        }

        // Verifica se carro 1 saiu do sistema (falta a margem)
        if pos_atual1
            > comprimento1
                + if via1 == 'H' {
                    VIAV_LARGURA
                } else {
                    VIAH_LARGURA
                }
        {
            break;
        }

        // Verifica se carro 2 saiu do sistema (falta a margem)
        if pos_atual2
            > comprimento2
                + if via2 == 'H' {
                    VIAV_LARGURA
                } else {
                    VIAH_LARGURA
                }
        {
            break;
        }
    }

    return false;
}

// Colisão de dois carros ao longo da mesma via
fn colisao_longitudinal(posicao_frente: f64, comprimento: f64, posicao_atras: f64) -> bool {
    return posicao_frente - comprimento <= posicao_atras;
}

// Detecta carro dentro do cruzamento
fn dentro_cruzamento(posicao: f64, comprimento: f64, via: char) -> bool {
    return posicao > 0.0
        && posicao
            <= comprimento
                + if via == 'H' {
                    VIAV_LARGURA
                } else {
                    VIAH_LARGURA
                };
}

fn main() {
    println!("Inicio do programa");
    simula_carros('H', ACELERACAO_MAXIMA / 10.0, 'H', ACELERACAO_MAXIMA);

    println!("Fim da simulação");
}
