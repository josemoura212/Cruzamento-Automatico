use std::thread::sleep;
use std::time::Duration;

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

const _VIAH_MARGEM: f64 = 15.0; //metros
const _VIAV_MARGEM: f64 = 15.0; //metros

const VIAH_LARGURA: f64 = 4.0; //metros
const VIAV_LARGURA: f64 = 4.0; //metros

const VIAH_PERIMETRO: f64 = 150.0; //metros
const VIAV_PERIMETRO: f64 = 150.0; //metros

const _CARRO_LARGURA: f64 = 2.0; //metros
const CARRO_COMPRIMENTO: f64 = 4.0; //metros

const VIA_MAXIMO_CARROS: usize = 4; // Número máximo de carros criados por via

// Velocidade de cruzeiro de qualquer veículo em metros por segundo
const VELOCIDADE_CRUZEIRO: f64 = 80.0 * (1000.0 / 3600.0);

// Velocidade máxima de qualquer veículo em metros por segundo
const VELOCIDADE_MAXIMA: f64 = 200.0 * (1000.0 / 3600.0);

// Aceleração máxima de qualquer veículo em metros por segundo ao quadrado
const ACELERACAO_MAXIMA: f64 = 3.0;

// Aceleração mínima de qualquer veículo em metros por segundo ao quadrado
const ACELERACAO_MINIMA: f64 = -10.0;

// Cruzamento entre duas vias
// 'enum' tem semântica 'move', mas 'Via' é barato e facilita poder clonar o valor às vezes
#[derive(Debug, Clone)]
enum Via {
    ViaH,
    ViaV,
}

// Descrição de um carro
struct Carro {
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

impl Carro {
    // Cria um novo carro
    fn new(placa: String, via: Via, acel: f64) -> Self {
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
                Via::ViaH => -VIAH_PERIMETRO,
                Via::ViaV => -VIAV_PERIMETRO,
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
    fn mostra(&self) {
        println!(
            "@{} na posição {:?}{}, velocidade {}, aceleração {}",
            self.placa, self.via, self.pos_atual, self.vel_atual, self.acel_atual
        );
    }

    // Avança o estado de um carro por tickms milissegundos
    fn tick(&mut self, tickms: f64) {
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

        //self.mostra();
    }
}

// Transito composto por carros nas vias
struct Transito {
    num_carros_criados_h: usize, // Número de carros criados para a via H
    num_carros_sairam_h: usize,  // Número de carros ativos na via H
    carros_via_h: [Carro; 4],    // Descrição dos carros criados até agora na via H
    num_carros_criados_v: usize, // Número de carros criados para a via V
    num_carros_sairam_v: usize,  // Número de carros ativos na via V
    carros_via_v: [Carro; 4],    // Descrição dos carros criados até agora na via V
}

impl Transito {
    // Cria um novo transito
    fn new() -> Self {
        Self {
            num_carros_criados_h: 0,
            num_carros_sairam_h: 0,
            carros_via_h: [
                Carro::new(String::from("AAA0000"), Via::ViaH, 0.0),
                Carro::new(String::from("AAA0000"), Via::ViaH, 0.0),
                Carro::new(String::from("AAA0000"), Via::ViaH, 0.0),
                Carro::new(String::from("AAA0000"), Via::ViaH, 0.0),
            ],
            num_carros_criados_v: 0,
            num_carros_sairam_v: 0,
            carros_via_v: [
                Carro::new(String::from("AAA0000"), Via::ViaV, 0.0),
                Carro::new(String::from("AAA0000"), Via::ViaV, 0.0),
                Carro::new(String::from("AAA0000"), Via::ViaV, 0.0),
                Carro::new(String::from("AAA0000"), Via::ViaV, 0.0),
            ],
        }
    }

    // Detecta se ocorreu uma colisão
    fn ocorreu_colisao(&self) -> Option<&str> {
        let mut i: usize = self.num_carros_sairam_h + 1;
        while i < self.num_carros_criados_h {
            if self.carros_via_h[i - 1].pos_atual - self.carros_via_h[i - 1].comprimento
                <= self.carros_via_h[i].pos_atual
            {
                return Some("Colisão via H");
            }
            i += 1;
        }

        i = self.num_carros_sairam_v + 1;
        while i < self.num_carros_criados_v {
            if self.carros_via_v[i - 1].pos_atual - self.carros_via_v[i - 1].comprimento
                <= self.carros_via_v[i].pos_atual
            {
                return Some("Colisão via V");
            }
            i += 1;
        }

        // Detecta colisão no cruzamento
        let mut cruzando_h = false;
        let mut cruzando_v = false;
        i = self.num_carros_sairam_h;
        while i < self.num_carros_criados_h {
            cruzando_h = cruzando_h
                || (self.carros_via_h[i].pos_atual > 0.0
                    && self.carros_via_h[i].pos_atual
                        < 0.0 + VIAV_LARGURA + self.carros_via_h[i].comprimento);
            i += 1;
        }
        i = self.num_carros_sairam_v;
        while i < self.num_carros_criados_v {
            cruzando_v = cruzando_v
                || (self.carros_via_v[i].pos_atual > 0.0
                    && self.carros_via_v[i].pos_atual
                        < 0.0 + VIAH_LARGURA + self.carros_via_v[i].comprimento);
            i += 1;
        }
        if cruzando_h && cruzando_v {
            return Some("Colisão dentro do cruzamento");
        }

        // Não tem colisão
        None
    }

    // Chega um novo carro no transito
    fn chega_carro(&mut self, via: Via, acel: f64) -> bool {
        let jah_tem = match via {
            Via::ViaH => self.num_carros_criados_h,
            Via::ViaV => self.num_carros_criados_v,
        };

        if jah_tem == VIA_MAXIMO_CARROS {
            return false;
        }

        let mut nova_placa = String::from("CCC");
        nova_placa.push_str(&format!("{:04}", jah_tem));
        let novo_carro = Carro::new(nova_placa, via.clone(), acel);

        match via {
            // Posso usar aqui pois foi usado um clone antes
            Via::ViaH => {
                self.carros_via_h[self.num_carros_criados_h] = novo_carro;
                self.num_carros_criados_h += 1;
            }
            Via::ViaV => {
                self.carros_via_v[self.num_carros_criados_v] = novo_carro;
                self.num_carros_criados_v += 1;
            }
        }

        true
    }

    // Avança o estado de todos os carros por tickms milissegundos
    fn tick(&mut self, tickms: f64) {
        println!("transito.tick");

        let mut i;

        i = self.num_carros_sairam_h;
        while i < self.num_carros_criados_h {
            self.carros_via_h[i].tick(tickms);
            i += 1;
        }

        i = self.num_carros_sairam_v;
        while i < self.num_carros_criados_v {
            self.carros_via_v[i].tick(tickms);
            i += 1;
        }

        // Carro mais antigo na via H saiu do sistema ?
        if self.num_carros_sairam_h < self.num_carros_criados_h {
            let mais_antigo_h = &self.carros_via_h[self.num_carros_sairam_h];
            if mais_antigo_h.pos_atual > mais_antigo_h.comprimento + VIAV_LARGURA {
                println!("@{} saiu da via H", mais_antigo_h.placa);
                self.num_carros_sairam_h += 1;
            }
        }

        // Carro mais antigo na via V saiu do sistema ?
        if self.num_carros_sairam_v < self.num_carros_criados_v {
            let mais_antigo_v = &self.carros_via_v[self.num_carros_sairam_v];
            if mais_antigo_v.pos_atual > mais_antigo_v.comprimento + VIAH_LARGURA {
                println!("@{} saiu da via H", mais_antigo_v.placa);
                self.num_carros_sairam_v += 1;
            }
        }
    }

    // Mostra estado das vias
    fn mostra_vias(&self) {
        println!("___Carros na via H___");
        let mut i = self.num_carros_sairam_h;
        while i < self.num_carros_criados_h {
            self.carros_via_h[i].mostra();
            i += 1;
        }

        println!("___Carros na via V___");
        i = self.num_carros_sairam_v;
        while i < self.num_carros_criados_v {
            self.carros_via_v[i].mostra();
            i += 1;
        }
    }
}

// Simula carros até saírem do perímetro ou colidirem
// Retorna se houve colisão ou não
fn simula_carros() {
    const TEMPO_ENTRE_CHEGADAS: f64 = 3000.0; // Tempo entre chegadas em ms

    // Cria uma descrição de trânsito
    let mut transito = Transito::new();

    // Cria o primeiro carro da via H
    transito.chega_carro(Via::ViaH, ACELERACAO_MAXIMA); // ACELERACAO_MINIMA para colidir em H

    // Cria o primeiro carro da via V
    transito.chega_carro(Via::ViaV, ACELERACAO_MAXIMA);

    // Tempo até a próxima chegada de um carro
    let mut tempo_ateh_proxima_chegada = TEMPO_ENTRE_CHEGADAS;

    println!("simula_carros");
    let mut tickms: f64; // tempo que passou em cada tick, milisegundos

    loop {
        // Passos de 100ms
        tickms = 100.0;

        // Ao final do tick devemos atualizar o estado do carro
        sleep(Duration::from_millis(tickms.round() as u64));
        transito.tick(tickms);

        // Mostra estado das vias
        transito.mostra_vias();

        // Aborta a simulação se ocorreu colisão
        match transito.ocorreu_colisao() {
            Some(m) => panic!("Ocorreu colisao: {}", m),
            None => {}
        }

        // Verifica se algum carro no sistema
        if transito.num_carros_criados_h == transito.num_carros_sairam_h
            && transito.num_carros_criados_v == transito.num_carros_sairam_v
        {
            break;
        }

        // Verifica se está na hora de chegar novos carros
        tempo_ateh_proxima_chegada -= tickms;

        if tempo_ateh_proxima_chegada <= 0.0 {
            let _acel: f64 = 0.0;
            assert!(
                transito.chega_carro(Via::ViaH, ACELERACAO_MAXIMA),
                "Falha em chegar um carro via H"
            );
            assert!(
                transito.chega_carro(Via::ViaV, ACELERACAO_MAXIMA),
                "Falha em chegar um carro via V"
            );
            tempo_ateh_proxima_chegada += TEMPO_ENTRE_CHEGADAS;
        }
    }
}

fn main() {
    println!("Inicio da simulação de cruzamento automático");
    simula_carros();
    println!("Fim da simulação de cruzamento automático");
}
