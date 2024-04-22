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

//use transito::Transito;
use crate::transito::Via;

struct Situacao {
    placa: String,    // placa deste carro
    via: Via,         // via deste carro
    acel_max: f64,    // metros por segundo ao quadrado
    acel_min: f64,    // metros por segundo ao quadrado
    vel_max: f64,     // metros por segundo
    comprimento: f64, // metros
    pos_atual: f64,   // metros do cruzamento
    vel_atual: f64,   // metros por segundo
    acel_atual: f64,  // metros por segundo ao quadrado
}

// Descreve um controlador
pub struct Controlador {
    situacao: HashMap<String, Situacao>,
}

// Cria um novo controlador
impl Controlador {
    // Cria um novo controlador
    pub fn new() -> Self {
        Self {
            situacao: HashMap::new(),
        }
    }

    // Ação periódica de controle
    pub fn controle(&mut self, comunicacao: &mut Comunicacao) {
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
                                Some(sit_veiculo) => {
                                    sit_veiculo.pos_atual = pos_atual;
                                    sit_veiculo.vel_atual = vel_atual;
                                    sit_veiculo.acel_atual = acel_atual;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Calcula ações de controle

        // Envia novas acelerações para os veículos

        // Solicita nova situação de todos
        for (placa, situacao) in &self.situacao {
            println!("#controlador solicita situacao de @{}", placa);
            let msg = MensagemDoControlador::PedeSituacao {
                placa: placa.to_string(),
            };
            comunicacao.send_por_controlador(placa.to_string(), msg);
        }
    }
}
