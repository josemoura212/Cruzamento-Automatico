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

//use std::time::Instant;	//Caso queira saber o tempo real

use crate::comunicacao::{Comunicacao, MensagemDeVeiculo, MensagemDoControlador};

use crate::transito::{self, Via};

mod faz_nada;
use faz_nada::FazNada;

mod semaforo;
use semaforo::Semaforo;

// Descreve funções exigidas de um Controlador implementado como submódulo
pub trait Controlador {
    // Cria um novo semáforo
    fn new(display_tudo: bool) -> Self;

    // Cálcula ações de controle
    fn estrategia(&mut self, tempo_decorrido: f64, situacao: &mut HashMap<String, Situacao>);
}

// Usado para definir o tipo de controlador
pub enum TipoControlador {
    SEMAFORO,
    FAZNADA,
}

// Usado neste módulo para armazenar o controlador usado
enum MeuControlador {
    SEMAFORO(Semaforo),
    FAZNADA(FazNada),
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
}

// Informações necessárias para realizar o controle
pub struct Controle {
    situacao: HashMap<String, Situacao>,
    controlador: MeuControlador,
    display_tudo: bool,
}

impl Controle {
    // Cria um novo controlador
    pub fn new(tipo: TipoControlador) -> Self {
        Self {
            situacao: HashMap::new(),
            controlador: match tipo {
                TipoControlador::SEMAFORO => MeuControlador::SEMAFORO(Semaforo::new(true)),
                TipoControlador::FAZNADA => MeuControlador::FAZNADA(FazNada::new(true)),
            },
            display_tudo: true,
        }
    }

    // Ação periódica de controle
    pub fn acao_controle(&mut self, tempo_decorrido: f64, comunicacao: &mut Comunicacao) {
        // Caso queira saber o tempo real
        // println!("-----------------Depois do sleep (s): {:?}", Instant::now()); !!!

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
                                pos_atual: match via {
                                    // Na entrada está longe !!!
                                    Via::ViaH => -transito::VIAH_PERIMETRO,
                                    Via::ViaV => -transito::VIAV_PERIMETRO,
                                },
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

        // Retira da 'situacao' veículos que já sairam do cruzamento	!!!
        let mut retirar: Vec<String> = Vec::new();
        for (_k, v) in self.situacao.iter() {
            let limite = match v.via {
                Via::ViaH => v.comprimento + transito::VIAV_LARGURA,
                Via::ViaV => v.comprimento + transito::VIAH_LARGURA,
            };
            if v.pos_atual > limite {
                retirar.push(v.placa.clone());
            }
        }

        for k in retirar {
            println!("#controlador retira da base de dados veículo @{}", k);
            self.situacao.remove(&k);
        }

        // Calcula as ações de controle
        match &mut self.controlador {
            MeuControlador::SEMAFORO(ss) => ss.estrategia(tempo_decorrido, &mut self.situacao),
            MeuControlador::FAZNADA(nn) => nn.estrategia(tempo_decorrido, &mut self.situacao),
        }

        // Envia novas acelerações para os veículos
        for (k, v) in &self.situacao {
            let msg = MensagemDoControlador::SetAcel {
                placa: k.to_string(),
                acel: v.acel_desejada,
            };
            comunicacao.send_por_controlador(k.to_string(), msg);

            if self.display_tudo {
                println!(
                    "#controlador setAceleracao de @{} em {:.2}",
                    k.to_string(),
                    v.acel_desejada
                );
            }
        }

        // Solicita nova situação de todos
        //for (placa, situacao) in &self.situacao {
        for placa in self.situacao.keys() {
            println!("#controlador solicita situacao de @{}", placa);
            let msg = MensagemDoControlador::PedeSituacao {
                placa: placa.to_string(),
            };
            comunicacao.send_por_controlador(placa.to_string(), msg);
        }
    }
}
