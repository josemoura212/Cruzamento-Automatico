use std::collections::HashMap;
use std::fmt::Debug;

use super::{Controlador, Situacao};

use crate::transito::{Via, VIAH_LARGURA};

use crate::transito::veiculos::VELOCIDADE_CRUZEIRO;

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
            tempo_verde: 13000.0,   // ms
            tempo_amarelo: 5000.0,  // ms
            restam_verde: 13000.0,  // ms
            restam_amarelo: 5000.0, // ms
            via_verde: Via::ViaH,
            via_vermelho: Via::ViaV,
            amarelo: false,
            display_tudo,
        }
    }

    // Cálcula ações de controle, tempos em milissegundos !!!
    fn estrategia(&mut self, tempo_decorrido: f64, situacao: &mut HashMap<String, Situacao>) {
        // Atualiza tempos de amarelo e verde, conforme o caso
        if self.amarelo {
            // Avança o tempo em amarelo
            self.restam_amarelo -= tempo_decorrido;
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
            self.restam_verde -= tempo_decorrido;
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
                "#SEM restam verde {:?} {:.2}   restam amarelo {:.2} {}    vermelho {:?}",
                self.via_verde,
                self.restam_verde,
                self.restam_amarelo,
                self.amarelo,
                self.via_vermelho
            );
        }

        // Monta uma lista ordenada para cada via
        #[derive(Debug)]
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
        let ordem_via_vermelho: &Vec<MiniSituacao>;
        let ordem_via_verde: &Vec<MiniSituacao>;
        if self.via_vermelho == Via::ViaH {
            ordem_via_vermelho = &ordem_via_h; // Basta um empréstimo/referência !!!
            ordem_via_verde = &ordem_via_v; // Basta um empréstimo/referência
        } else {
            ordem_via_vermelho = &ordem_via_v; // Basta um empréstimo/referência
            ordem_via_verde = &ordem_via_h; // Basta um empréstimo/referência
        }

        // Ações para veículos na via vermelha

        // Primeiro carro vai até 'um espaçamento' antes do cruzamento
        // Demais ficam sempre 'um espaçamento' atrás do anterior na via
        let espacamento = 4.0; // metros
        let mut pos_alvo = -espacamento;

        for mini in ordem_via_vermelho {
            let veiculo = situacao.get_mut(&mini.placa).unwrap(); // Sei que a placa existe

            if veiculo.pos_atual > 0.0 {
                veiculo.acel_desejada = veiculo.vel_max;
            } else if veiculo.pos_atual >= pos_alvo {
                if veiculo.vel_atual <= 0.0005 {
                    veiculo.acel_desejada = 0.0;
                } else {
                    veiculo.acel_desejada = veiculo.acel_min;
                }
            } else {
                veiculo.acel_desejada =
                    veiculo.vel_atual.powi(2) / (2.0 * (veiculo.pos_atual - pos_alvo));
            }

            if self.display_tudo {
                println!(
                    "#SEM @{}  atual:{:.2}  alvo:{:.2}  acel:{:.2}->{:.2}",
                    veiculo.placa,
                    veiculo.pos_atual,
                    pos_alvo,
                    veiculo.acel_atual,
                    veiculo.acel_desejada
                );
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

                let tpassar = (0.0 + veiculo.comprimento + VIAH_LARGURA - veiculo.pos_atual)
                    / veiculo.vel_atual;
                if self.display_tudo {
                    println!(
                        "#SEM @{}, {:?} em amarelo, t/passar {:.2}",
                        veiculo.placa,
                        self.via_verde,
                        1000.0 * tpassar
                    );
                }

                // Calcula a aceleracao com base no passa/nao-passa no amarelo
                if 1000.0 * tpassar <= self.restam_amarelo * 0.9 {
                    veiculo.acel_desejada = veiculo.acel_max;
                } else {
                    veiculo.acel_desejada = veiculo.vel_atual.powi(2) / (2.0 * (veiculo.pos_atual));
                }

                // Se vai parar
                if veiculo.acel_desejada <= 0.0
                    && veiculo.pos_atual >= pos_alvo
                    && veiculo.vel_atual <= 0.0005
                {
                    veiculo.acel_desejada = 0.0;
                }
            }
        } else {
            // Ações para veículos na via verde sem amarelo
            for i in 0..ordem_via_verde.len() {
                let mini = &ordem_via_verde[i];

                if i == 0 {
                    // Primeiro: acelera livremente
                    let veiculo = situacao.get_mut(&mini.placa).unwrap(); // Sei que a placa existe

                    if veiculo.vel_atual > 1.1 * VELOCIDADE_CRUZEIRO {
                        veiculo.acel_desejada = veiculo.acel_min;
                    } else if veiculo.vel_atual < 0.9 * VELOCIDADE_CRUZEIRO {
                        veiculo.acel_desejada = veiculo.acel_max;
                    } else {
                        veiculo.acel_desejada = 0.0;
                    }
                    if self.display_tudo {
                        println!(
                            "#SEM @{}, {:?} em verde, vel {:.2}, acel {:.2}",
                            veiculo.placa, self.via_verde, veiculo.vel_atual, veiculo.acel_desejada
                        );
                    }
                } else {
                    // Veículos seguintes: acelerar mas sem bater no da frente !!!

                    let afrente = situacao.get(&ordem_via_verde[i - 1].placa).unwrap(); // Sei que a placa existe
                    let afrente_pos = afrente.pos_atual;
                    let afrente_vel = afrente.vel_atual;
                    let afrente_comp = afrente.comprimento; // empréstimo de 'situacao' p/ afrente termina aqui

                    let veiculo = situacao.get_mut(&mini.placa).unwrap(); // Sei que a placa existe

                    if veiculo.vel_atual > 1.1 * VELOCIDADE_CRUZEIRO {
                        veiculo.acel_desejada = veiculo.acel_min;
                    } else if veiculo.vel_atual < 0.9 * VELOCIDADE_CRUZEIRO {
                        veiculo.acel_desejada = veiculo.acel_max;
                    } else {
                        #[allow(clippy::no_effect)]
                        veiculo.acel_desejada;
                    }

                    // Supõe pista livre
                    if veiculo.vel_atual > 1.1 * VELOCIDADE_CRUZEIRO {
                        veiculo.acel_desejada = veiculo.acel_min;
                    } else if veiculo.vel_atual < 0.9 * VELOCIDADE_CRUZEIRO {
                        veiculo.acel_desejada = veiculo.acel_max;
                    } else {
                        veiculo.acel_desejada = 0.0;
                    }

                    // Distância em tempo do veículo da frente
                    let delta_t_atual = 1000.0 * (afrente_pos - afrente_comp - veiculo.pos_atual)
                        / veiculo.vel_atual;
                    if delta_t_atual < 2000.0 {
                        // Distância de segurança é 2s
                        veiculo.acel_desejada = veiculo.acel_min / 2.0;
                    } else if veiculo.vel_atual > afrente_vel {
                        let delta_t_colisao = 1000.0
                            * (afrente_pos - afrente_comp - veiculo.pos_atual)
                            / (veiculo.vel_atual - afrente_vel);
                        if delta_t_colisao < 10000.0 {
                            // Colisão em menos de 10s
                            veiculo.acel_desejada = veiculo.acel_min / 2.0;
                        }
                    }

                    if self.display_tudo {
                        println!(
                            "#SEM @{}, {:?} em verde, vel {:.2}, delta_t {:.2}, acel {:.2}",
                            veiculo.placa,
                            self.via_verde,
                            veiculo.vel_atual,
                            delta_t_atual,
                            veiculo.acel_desejada
                        );
                    }
                }
            }
        }
    }
}
