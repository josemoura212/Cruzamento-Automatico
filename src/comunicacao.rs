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

use std::collections::{HashMap, VecDeque};

use crate::transito::Via;

// Tipos de mensagens enviadas por veículos para o controlador
pub enum MensagemDeVeiculo {
    Chegada {
        placa: String,
        via: Via,
        acel_max: f64,
        acel_min: f64,
        vel_max: f64,
        comprimento: f64,
    }, // Informa que chegou
    SituacaoAtual {
        placa: String,
        pos_atual: f64,
        vel_atual: f64,
        acel_atual: f64,
    }, // Informa a sua situação
}

// Tipos de mensagens enviadas pelo controlador para veículos
pub enum MensagemDoControlador {
    SetAcel { placa: String, acel: f64 }, // Determina a nova aceleração
    PedeSituacao { placa: String },       // Pede a situação
}

// Sistema de comunicação entre veículos e controlador
pub struct Comunicacao {
    mensagens_de_veiculo: Vec<MensagemDeVeiculo>,
    mensagens_do_controlador: HashMap<String, VecDeque<MensagemDoControlador>>,
}

impl Comunicacao {
    // Cria um novo sistema de comunicação
    pub fn new() -> Self {
        Self {
            mensagens_de_veiculo: Vec::new(),
            mensagens_do_controlador: HashMap::new(),
        }
    }

    // Permite um veículo enviar mensagens
    pub fn send_por_veiculo(&mut self, msg: MensagemDeVeiculo) {
        self.mensagens_de_veiculo.push(msg);
    }

    // Permite o controlador enviar mensagens
    pub fn send_por_controlador(&mut self, placa: String, msg: MensagemDoControlador) {
        let lista = self
            .mensagens_do_controlador
            .entry(placa)
            .or_insert(VecDeque::new());
        lista.push_back(msg);
    }

    // Permite um veículo receber uma mensagem vinda do controlador
    pub fn receive_por_veiculo(&mut self, placa: &String) -> Option<MensagemDoControlador> {
        match self.mensagens_do_controlador.get_mut(placa) {
            None => None,
            Some(x) => x.pop_front(),
        }
    }

    // Permite ao controlador receber uma mensagem vinda de veículo
    pub fn receive_por_controlador(&mut self) -> Option<MensagemDeVeiculo> {
        if self.mensagens_de_veiculo.len() == 0 {
            Option::None
        } else {
            Option::Some(self.mensagens_de_veiculo.swap_remove(0))
        }
    }
}
