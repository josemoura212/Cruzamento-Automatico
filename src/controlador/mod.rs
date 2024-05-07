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

use std::time::Instant;

use crate::comunicacao::{Comunicacao, MensagemDeVeiculo, MensagemDoControlador};

use crate::transito::{self, Via};

mod faz_nada;
use faz_nada::FazNada;

mod semaforo;
use semaforo::Semaforo;

const TEMPO_ENTRE_CONTROLES: f64 = 500.0; // Tempo entre ações de controle, em ms

// Descreve funções exigidas de um Controlador implementado como submódulo
pub trait Controlador {
    // Cria um novo semáforo
    fn new(display_tudo: bool) -> Self;

    // Cálcula ações de controle
    fn estrategia(&mut self, tempo_decorrido: f64, situacao: &mut HashMap<String, Situacao>);
}

// Usado para definir o tipo de controlador
pub enum TipoControlador {
    Semaforo,
    FazNada,
}

// Usado neste módulo para armazenar o controlador usado
enum MeuControlador {
    Semaforo(Semaforo),
    FazNada(FazNada),
}

// Descreve a situação de um veículo em particular
#[derive(Debug)]
pub struct Situacao {
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
    estou_vivo: i32,    // recarrega quando tem comunicação
}

// Informações necessárias para realizar o controle
pub struct Controle {
    situacao: HashMap<String, Situacao>,
    controlador: MeuControlador,
    display_tudo: bool,
    tempo_ateh_proxima_solicitacao: f64,
    tempo_ateh_proxima_estrategia: f64,
}

impl Controle {
    // Cria um novo controlador
    pub fn new(tipo: TipoControlador) -> Self {
        Self {
            situacao: HashMap::new(),
            controlador: match tipo {
                TipoControlador::Semaforo => MeuControlador::Semaforo(Semaforo::new(true)),
                TipoControlador::FazNada => MeuControlador::FazNada(FazNada::new(true)),
            },
            display_tudo: true,
            tempo_ateh_proxima_solicitacao: TEMPO_ENTRE_CONTROLES - 100.0,
            tempo_ateh_proxima_estrategia: TEMPO_ENTRE_CONTROLES,
        }
    }

    // Ação periódica de controle
    pub fn acao_controle(&mut self, tempo_decorrido: f64, comunicacao: &mut Comunicacao) {
        // Processa as mensagens recebidas em todos os ciclos
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
                                pos_atual: match via {
                                    // Na entrada está longe !!!
                                    Via::ViaH => -transito::VIAH_PERIMETRO,
                                    Via::ViaV => -transito::VIAV_PERIMETRO,
                                },
                                vel_atual: 0.0,
                                acel_atual: 0.0,
                                acel_desejada: 0.0,
                                estou_vivo: 2,
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
                                    veiculo.estou_vivo = 2;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Solicita nova situação de todos os veículos conhecidos
        self.tempo_ateh_proxima_solicitacao -= tempo_decorrido;
        if self.tempo_ateh_proxima_solicitacao <= 0.0 {
            self.tempo_ateh_proxima_solicitacao += TEMPO_ENTRE_CONTROLES;
            for placa in self.situacao.keys() {
                // só precisa das chaves
                println!("#controlador solicita situacao de @{}", placa);
                let msg = MensagemDoControlador::PedeSituacao {
                    placa: placa.to_string(),
                };
                comunicacao.send_por_controlador(placa.to_string(), msg);
            }
        }

        // Se está na hora:
        //		(1) Retira da 'situacao' veículos que já sairam do cruzamento
        // 		(2) Calcula as ações de controle
        // 		(3) Envia novas acelerações para os veículos
        self.tempo_ateh_proxima_estrategia -= tempo_decorrido;
        if self.tempo_ateh_proxima_estrategia <= 0.0 {
            self.tempo_ateh_proxima_estrategia += TEMPO_ENTRE_CONTROLES;
            println!("#controlador: depois do sleep (s): {:?}", Instant::now());

            // (1) Retira da 'situacao' veículos que já sairam do cruzamento
            let mut retirar: Vec<String> = Vec::new();
            for (_k, v) in self.situacao.iter_mut() {
                v.estou_vivo -= 1;
                if v.estou_vivo == 0 {
                    retirar.push(v.placa.clone());
                }
            }
            for k in retirar {
                println!("#controlador retira da base de dados veículo @{}", k);
                self.situacao.remove(&k);
            }

            // (2) Calcula as ações de controle
            match &mut self.controlador {
                MeuControlador::Semaforo(ss) => {
                    ss.estrategia(TEMPO_ENTRE_CONTROLES, &mut self.situacao)
                }
                MeuControlador::FazNada(nn) => {
                    nn.estrategia(TEMPO_ENTRE_CONTROLES, &mut self.situacao)
                }
            }

            // (3) Envia novas acelerações para os veículos
            for (k, v) in &self.situacao {
                let msg = MensagemDoControlador::SetAcel {
                    placa: k.to_string(),
                    acel: v.acel_desejada,
                };
                comunicacao.send_por_controlador(k.to_string(), msg);

                if self.display_tudo {
                    println!(
                        "#controlador setAceleracao de @{} em {:.2}",
                        k, v.acel_desejada
                    );
                }
            }
        }
    }
}
