use crate::comunicacao::{Comunicacao, MensagemDeVeiculo, MensagemDoControlador};

use super::Via;

pub const _CARRO_LARGURA: f64 = 2.0; //metros
pub const CARRO_COMPRIMENTO: f64 = 4.0; //metros

// Velocidade de cruzeiro de qualquer veículo em metros por segundo
pub const VELOCIDADE_CRUZEIRO: f64 = 80.0 * (1000.0 / 3600.0);

// Velocidade máxima de qualquer veículo em metros por segundo
pub const VELOCIDADE_MAXIMA: f64 = 200.0 * (1000.0 / 3600.0);

// Aceleração máxima de qualquer veículo em metros por segundo ao quadrado
pub const ACELERACAO_MAXIMA: f64 = 3.0;

// Aceleração mínima de qualquer veículo em metros por segundo ao quadrado
pub const ACELERACAO_MINIMA: f64 = -10.0;

// Descrição de um carro
pub struct Carro {
    pub placa: String,    // placa deste carro
    pub via: Via,         // via deste carro
    pub acel_max: f64,    // metros por segundo ao quadrado
    pub acel_min: f64,    // metros por segundo ao quadrado
    pub vel_max: f64,     // metros por segundo
    pub comprimento: f64, // metros
    pub pos_atual: f64,   // metros do cruzamento
    pub vel_atual: f64,   // metros por segundo
    pub acel_atual: f64,  // metros por segundo ao quadrado
}

impl Carro {
    // Cria um novo carro
    pub fn new(placa: String, via: Via, acel: f64) -> Self {
        let (res, msg) = Carro::valida_placa(&placa);
        assert!(res, "   Placa inválida: {} @{}", msg, placa);

        assert!(
            acel >= ACELERACAO_MINIMA && acel <= ACELERACAO_MAXIMA,
            "   Aceleração inválida: {} {}",
            placa,
            acel
        );

        Self {
            placa,
            via: via.clone(),
            acel_max: ACELERACAO_MAXIMA,
            acel_min: ACELERACAO_MINIMA,
            vel_max: VELOCIDADE_MAXIMA,
            comprimento: CARRO_COMPRIMENTO,
            pos_atual: match via {
                // Posso usar aqui pois foi clonado antes
                Via::ViaH => -super::VIAH_PERIMETRO,
                Via::ViaV => -super::VIAV_PERIMETRO,
            },
            vel_atual: VELOCIDADE_CRUZEIRO,
            acel_atual: acel,
        }
    }

    // Valida formato de uma placa
    fn valida_placa(placa: &str) -> (bool, &str) {
        // Só aceita caracteres ASCII
        if !placa.is_ascii() {
            return (false, "Placa não é ASCII");
        }
        // Só aceita placa velha
        if placa.len() != 7 {
            return (false, "Placa não tem o tamanho certo");
        }
        // Valida parte das letras
        let inicio = &placa[0..3];
        for x in inicio.chars() {
            if !x.is_alphabetic() {
                return (false, "Placa não tem letras no início");
            }
        }
        // Valida parte dos números
        let fim = &placa[3..];
        for x in fim.chars() {
            if !x.is_ascii_digit() {
                return (false, "Placa não tem dígitos no final");
            }
        }
        (true, "")
    }

    // Mostra o estado de um carro na tela
    pub fn mostra(&self) {
        println!(
            "   @{} na posição {:?} {:.3}, velocidade {:.2}, aceleração {:.2}",
            self.placa, self.via, self.pos_atual, self.vel_atual, self.acel_atual
        );
    }

    // Avança o estado de um carro por tickms milissegundos
    pub fn tick(&mut self, tickms: f64, comunicacao: &mut Comunicacao) {
        //self.mostra();

        let pos_anterior = self.pos_atual;

        self.pos_atual = self.pos_atual
            + self.vel_atual * (tickms / 1000.0)
            + self.acel_atual * (tickms / 1000.0) * (tickms / 1000.0) / 2.0;

        self.vel_atual = self.vel_atual + self.acel_atual * (tickms / 1000.0);

        // Restrições de um carro
        if self.pos_atual < pos_anterior {
            // Não anda para tras
            self.pos_atual = pos_anterior;
        }

        if self.vel_atual < 0.0 {
            // Não anda para tras
            self.vel_atual = 0.0;
        }

        if self.vel_atual > self.vel_max {
            self.vel_atual = self.vel_max; // Trava na velocidade máxima
        }

        // Processa as mensagens recebidas por este carro
        loop {
            match comunicacao.receive_por_veiculo(&self.placa) {
                None => break,
                Some(msg) => {
                    match msg {
                        MensagemDoControlador::SetAcel { placa, acel } => {
                            println!("#veiculo @{} recebe acel {:.2}", placa, acel);
                            // Veículo só aceita aceleração válida !!!
                            if acel > self.acel_max {
                                self.acel_atual = self.acel_max;
                            } else if acel < self.acel_min {
                                self.acel_atual = self.acel_min;
                            } else {
                                self.acel_atual = acel
                            }
                        }

                        MensagemDoControlador::PedeSituacao { placa } => {
                            println!("#veiculo @{} informa sua situacao", &self.placa);
                            let msg = MensagemDeVeiculo::SituacaoAtual {
                                placa: placa,
                                pos_atual: self.pos_atual,
                                vel_atual: self.vel_atual,
                                acel_atual: self.acel_atual,
                            };
                            comunicacao.send_por_veiculo(msg);
                        }
                    }
                }
            }
        }
    }
}
