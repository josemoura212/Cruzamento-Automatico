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

use std::collections::HashMap;

use crate::comunicacao::{Comunicacao, MensagemDeVeiculo, MensagemDoControlador};

use crate::transito::Via;

// Algoritmo de controle que imita um semáforo
struct Semaforo {
    tempo_verde: f64,   // tempo de Verde em s
    tempo_amarelo: f64, // tempo de Amarelo em s
    restam_verde: f64,  // tempo que resta nesta fase
    restam_amarelo: f64,

    via_verde: Via,    // qual via esta verde ou amarelo
    via_vermelho: Via, // qual via esta vermelho

    amarelo: bool,      // Se amarelo está ligado
    display_tudo: bool, // Mostra todas as informações na tela
}

impl Semaforo {
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

    // Cálcula ações de controle, trabalha em segundos
    fn estrategia(&mut self, ms_decorrido: f64, situacao: &mut HashMap<String, Situacao>) {
        let tempo_decorrido = ms_decorrido / 1000.0;
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
                "nVerde {:?} {}    nVermelho {:?}    restam_amarelo {}",
                self.via_verde, self.restam_verde, self.via_vermelho, self.restam_amarelo
            );
        }

        // Em 'situacao: &mut HashMap<String,Situacao>' carros não estão nem separados nem ordenados
        // Precisam estar separados pela via, ordenados pela posição
        // Monta uma lista ordenada pela posição para cada via
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

        /*	Ordenação:
            Maneira usual seria esta:
        let mut vv = vec![1, 5, 10, 2, 15];
        vv.sort();
        println!("{:?}",vv);
        //'vv.sort_unstable()' é mais rápido que 'vv.sort()' mas não preserva a ordem dos iguais

        Problema:
            A existência do NaN em floats complica as coisas, não é possível usar 'sort()'
            Necessário usar o método 'partial_cmp()'

            let mut vv = vec![1.0, 5.0, 10.0, 2.0, 15.0];
            vv.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap() );
            println!("{:?}",vv);
            NaN geraria None, então unwrap() geraria pânico, mas não temos NaN no nosso Vec
            tente: let mut vv = vec![1.0, 5.0, 10.0, 2.0, 15.0, 0.0/0.0];
            */

        ordem_via_h.sort_unstable_by(|a, b| b.pos_atual.partial_cmp(&a.pos_atual).unwrap());
        ordem_via_v.sort_unstable_by(|a, b| b.pos_atual.partial_cmp(&a.pos_atual).unwrap());

        //println!("{:?}",ordem_via_h);

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

                // Calcula o tempo que leva para chegar ao cruzamento
                let t_chegar = (0.0 - veiculo.pos_atual) / veiculo.vel_atual;
                if self.display_tudo {
                    println!("@@@ {:?} @ t_chegar {}", self.via_verde, t_chegar);
                }

                // Calcula a aceleração com base no passa/nao-passa
                if t_chegar <= self.restam_amarelo {
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

// Descreve a situação de um veículo em particular
struct Situacao {
    placa: String,      // placa deste carro
    via: Via,           // via deste carro
    acel_max: f64,      // metros por segundo ao quadrado
    acel_min: f64,      // metros por segundo ao quadrado
    vel_max: f64,       // metros por segundo
    comprimento: f64,   // metros
    pos_atual: f64,     // metros do cruzamento
    vel_atual: f64,     // metros por segundo
    acel_atual: f64,    // metros por segundo ao quadrado
    acel_desejada: f64, // aceleração desejada pelo controle, metros por segundo ao quadrado
}

// Descreve um controlador
pub struct Controlador {
    situacao: HashMap<String, Situacao>,
    semaforo: Semaforo,
    display_tudo: bool,
}

impl Controlador {
    // Cria um novo controlador
    pub fn new() -> Self {
        Self {
            situacao: HashMap::new(),
            semaforo: Semaforo::new(true),
            display_tudo: true,
        }
    }

    // Ação periódica de controle
    pub fn controle(&mut self, tempo_decorrido: f64, comunicacao: &mut Comunicacao) {
        // Processa as mensagens recebidas
        loop {
            match comunicacao.receive_por_controlador() {
                None => break,
                Some(msg) => {
                    match msg {
                        MensagemDeVeiculo::Chegada {
                            placa,
                            via,
                            acel_max,
                            acel_min,
                            vel_max,
                            comprimento,
                        } => {
                            let novo = Situacao {
                                placa,
                                via,
                                acel_max,
                                acel_min,
                                vel_max,
                                comprimento,
                                pos_atual: 0.0,
                                vel_atual: 0.0,
                                acel_atual: 0.0,
                                acel_desejada: 0.0,
                            };
                            self.situacao.insert(novo.placa.clone(), novo);
                        }

                        MensagemDeVeiculo::SituacaoAtual {
                            placa,
                            pos_atual,
                            vel_atual,
                            acel_atual,
                        } => {
                            //localiza a situacao deste
                            let velho = self.situacao.get_mut(&placa);
                            match velho {
                                None => (),
                                Some(veiculo) => {
                                    veiculo.pos_atual = pos_atual;
                                    veiculo.vel_atual = vel_atual;
                                    veiculo.acel_atual = acel_atual;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Calcula as ações de controle
        self.semaforo
            .estrategia(tempo_decorrido, &mut self.situacao);

        // Envia novas acelerações para os veículos
        for (k, v) in &self.situacao {
            let msg = MensagemDoControlador::SetAcel {
                placa: k.to_string(),
                acel: v.acel_desejada,
            };
            comunicacao.send_por_controlador(k.to_string(), msg);

            if self.display_tudo {
                println!(
                    "#controlador setAceleracao de @{} em {}",
                    k.to_string(),
                    v.acel_desejada
                );
            }
        }

        // Solicita nova situação de todos
        for (placa, _situacao) in &self.situacao {
            println!("#controlador solicita situacao de @{}", placa);
            let msg = MensagemDoControlador::PedeSituacao {
                placa: placa.to_string(),
            };
            comunicacao.send_por_controlador(placa.to_string(), msg);
        }
    }
}
