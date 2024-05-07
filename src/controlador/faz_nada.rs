use std::collections::HashMap;

use super::{Controlador, Situacao};

// Algoritmo de controle que não faz nada
pub struct FazNada {
    display_tudo: bool,
}

impl Controlador for FazNada {
    // Cria um novo faz nada
    fn new(display_tudo: bool) -> Self {
        Self { display_tudo }
    }

    // Cálcula ações de controle
    fn estrategia(&mut self, tempo_decorrido: f64, situacao: &mut HashMap<String, Situacao>) {
        if self.display_tudo {
            println!("FazNada tempo decorrido {}", tempo_decorrido);
        }

        // Monta uma lista ordenada para as duas vias
        struct MiniSituacao {
            placa: String,  // placa deste carro
            pos_atual: f64, // metros do cruzamento
        }

        let mut ordem_duas_vias: Vec<MiniSituacao> = Vec::new();

        // Versão mais eficiente
        for (key, val) in situacao.iter_mut() {
            val.acel_desejada = 0.0;
            println!("key: {}    val: {:?}", key, val);
        }

        // Versão copiada e simplificada do Semáforo
        for (key, val) in situacao.iter() {
            ordem_duas_vias.push(MiniSituacao {
                placa: val.placa.clone(),
                pos_atual: val.pos_atual,
            });
            println!("key: {}    val: {:?}", key, val);
        }
        ordem_duas_vias.sort_unstable_by(|a, b| b.pos_atual.partial_cmp(&a.pos_atual).unwrap());
        // Ações para veículos nas duas vias
        for mini in ordem_duas_vias {
            let veiculo = situacao.get_mut(&mini.placa).unwrap(); // Sei que a placa existe
            veiculo.acel_desejada = 0.0;
        }
    }
}
