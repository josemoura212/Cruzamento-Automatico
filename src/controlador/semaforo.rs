use std::collections::HashMap;

use super::{Controlador, Situacao};

use crate::transito::Via;

// Algoritmo de controle que imita um semáforo
pub struct Semaforo {
    tempo_verde: f64,   // tempo de Verde em s
    tempo_amarelo: f64, // tempo de Amarelo em s

    restam_verde: f64, // tempo que resta nesta fase
    restam_amarelo: f64,

    via_verde: Via,    // qual via esta verde ou amarelo
    via_vermelho: Via, // qual via esta vermelho

    amarelo: bool, // Se amarelo está ligado
    display_tudo: bool,
}

impl Controlador for Semaforo {
    // Cria um novo semáforo
    fn new(display_tudo: bool) -> Self {
        Self {
            tempo_verde: 13.0,   // segundos
            tempo_amarelo: 5.0,  // segundos
            restam_verde: 13.0,  // segundos
            restam_amarelo: 5.0, // segundos
            via_verde: Via::ViaH,
            via_vermelho: Via::ViaV,
            amarelo: false,
            display_tudo,
        }
    }

    // Cálcula ações de controle
    fn estrategia(&mut self, tempo_decorrido: f64, situacao: &mut HashMap<String, Situacao>) {
        // Atualiza tempos de amarelo e verde, conforme o caso
        if self.amarelo {
            // Avança o tempo em amarelo
            self.restam_amarelo -= tempo_decorrido / 1000.0;
            if self.restam_amarelo > 0.0 {
                // Continua como estava
            } else {
                // Acabou o amarelo
                self.amarelo = false;
                // Troca via do verde
                if self.via_verde == Via::ViaH {
                    self.via_verde = Via::ViaV;
                    self.via_vermelho = Via::ViaH;
                } else {
                    self.via_verde = Via::ViaH;
                    self.via_vermelho = Via::ViaV;
                }
                // Inicia novo tempo de verde
                self.restam_amarelo = 0.0;
                self.restam_verde = self.tempo_verde;
            }
        } else {
            // Avança o tempo em verde
            self.restam_verde -= tempo_decorrido / 1000.0;
            if self.restam_verde > 0.0 {
                // Continua como estava
            } else {
                // Vai começar o amarelo deste verde
                self.amarelo = true;
                // Inicia novo tempo de amarelo
                self.restam_verde = 0.0;
                self.restam_amarelo = self.tempo_amarelo;
            }
        }

        if self.display_tudo {
            println!(
                "nVerde {:?} {}    nVermelho {:?}    restam_amarelo {}",
                self.via_verde, self.restam_verde, self.via_vermelho, self.restam_amarelo
            );
        }

        // Monta uma lista ordenada para cada via
        struct MiniSituacao {
            placa: String,  // placa deste carro
            pos_atual: f64, // metros do cruzamento
        }

        let mut ordem_via_h: Vec<MiniSituacao> = Vec::new();
        let mut ordem_via_v: Vec<MiniSituacao> = Vec::new();

        let situacao_iter = situacao.iter();
        for (_k, v) in situacao_iter {
            if v.via == Via::ViaH {
                ordem_via_h.push(MiniSituacao {
                    placa: v.placa.clone(),
                    pos_atual: v.pos_atual,
                });
            } else {
                ordem_via_v.push(MiniSituacao {
                    placa: v.placa.clone(),
                    pos_atual: v.pos_atual,
                });
            }
        }

        ordem_via_h.sort_unstable_by(|a, b| b.pos_atual.partial_cmp(&a.pos_atual).unwrap());
        ordem_via_v.sort_unstable_by(|a, b| b.pos_atual.partial_cmp(&a.pos_atual).unwrap());

        // Chama de via_vermelho e via_verde
        let ordem_via_vermelho: Vec<MiniSituacao>;
        let ordem_via_verde: Vec<MiniSituacao>;
        if self.via_vermelho == Via::ViaH {
            ordem_via_vermelho = ordem_via_h;
            ordem_via_verde = ordem_via_v;
        } else {
            ordem_via_vermelho = ordem_via_v;
            ordem_via_verde = ordem_via_h;
        }

        // Ações para veículos na via vermelha

        // Primeiro carro vai até '1 espaçamento' antes do cruzamento
        // Demais ficam sempre '1 espaçamento' atrás do anterior na via
        let espacamento = 4.0; // metros
        let mut pos_alvo = -espacamento;

        for mini in ordem_via_vermelho {
            let veiculo = situacao.get_mut(&mini.placa).unwrap(); // Sei que a placa existe

            if veiculo.pos_atual >= pos_alvo {
                veiculo.acel_desejada = veiculo.acel_min;
            } else {
                veiculo.acel_desejada =
                    veiculo.vel_atual.powi(2) / (2.0 * (pos_alvo - veiculo.pos_atual));
                if veiculo.acel_desejada < veiculo.acel_min {
                    veiculo.acel_desejada = veiculo.acel_min;
                }
            }

            pos_alvo -= veiculo.comprimento + espacamento;
        }

        // Ações para veículos na via verde com amarelo

        if self.amarelo {
            pos_alvo = -espacamento; // alguns em amarelo vão parar antes do cruzamento

            for mini in ordem_via_verde {
                let veiculo = situacao.get_mut(&mini.placa).unwrap(); // Sei que a placa existe

                // Problema: saber se os carros conseguem passar no amarelo.
                // Para isto, compara-se o tempo estimado de percurso ateh o cruzamento
                // e compara-se com o tempo restante.  Se for menor, acelerar para
                // passar.  Senao, parar.

                // Calcula o tempo e a posição quando se atinge a velocidade máxima
                // Supõe que está na velocidade máxima
                let t_vmax = 0.0;
                let pos_vmax = veiculo.pos_atual;
                if veiculo.vel_atual < veiculo.vel_max {
                    let t_vmax = (veiculo.vel_max - veiculo.vel_atual) / veiculo.acel_max;
                    let _pos_vmax = veiculo.pos_atual
                        + veiculo.vel_atual * t_vmax
                        + veiculo.acel_max * t_vmax * t_vmax / 2.0;
                }

                // Calcula o tempo que leva para passar pelo cruzamento
                let t_passar = t_vmax - pos_vmax / veiculo.vel_max;
                if self.display_tudo {
                    println!("@@@ {:?} @ tPassar {}", self.via_verde, t_passar);
                }

                // Calcula a aceleração com base no passa/nao-passa
                if t_passar <= self.restam_amarelo {
                    // passa
                    veiculo.acel_desejada = veiculo.acel_max;
                } else {
                    // não passa
                    veiculo.acel_desejada =
                        veiculo.vel_atual.powi(2) / (pos_alvo - veiculo.pos_atual) / 2.0;

                    if veiculo.acel_desejada < veiculo.acel_min {
                        veiculo.acel_desejada = veiculo.acel_min;
                    }

                    pos_alvo -= veiculo.comprimento + espacamento;
                }
            }
        } else {
            // Ações para veículos na via verde sem amarelo
            for i in 0..ordem_via_verde.len() {
                let mini = &ordem_via_verde[i];

                if i == 0 {
                    // Primeiro: acelerar até o máximo
                    let veiculo = situacao.get_mut(&mini.placa).unwrap(); // Sei que a placa existe
                    veiculo.acel_desejada = veiculo.acel_max;
                } else {
                    // Veículos seguintes: acelerar mas sem bater no da frente
                    let afrente = situacao.get(&ordem_via_verde[i - 1].placa).unwrap(); // Sei que a placa existe
                    let afrente_pos = afrente.pos_atual;
                    let afrente_vel = afrente.vel_atual;
                    let afrente_comp = afrente.comprimento; // empréstimo de 'situacao' p/ afrente termina aqui

                    let veiculo = situacao.get_mut(&mini.placa).unwrap(); // Sei que a placa existe
                    let dist = afrente_pos - veiculo.pos_atual;

                    let delta_v = afrente_vel - veiculo.vel_atual;
                    if delta_v > 0.0 {
                        veiculo.acel_desejada = veiculo.acel_max;
                    } else {
                        if dist <= espacamento {
                            veiculo.acel_desejada = veiculo.acel_min;
                        } else {
                            veiculo.acel_desejada =
                                -delta_v * delta_v / (2.0 * (dist - afrente_comp));
                        }
                    }
                }
            }
        }
    }
}
