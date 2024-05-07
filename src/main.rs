/* 	Projeto do Cruzamento Automático - versão s11 a04

*/

use core::f32;
use std::thread::sleep;
use std::time::Duration;

use rand::Rng; // Para gerar números aleatórios, não é 'std::'
               // Requer [dependencies] rand = "0.8.5"

use std::env; // Para acessar os argumentos da linha de comando, exemplo:
              // cargo run -- s/n/o 2.0 3.0
              /*
              use device_query::{DeviceQuery, DeviceState, Keycode};	// Para acessar o teclado, não é 'std::'
                                                                      // Requer [dependencies] device_query = "1.1.3"
              */

use speedy2d::color::Color; // Para gráficos
use speedy2d::dimen::Vector2;
use speedy2d::font::{Font, TextLayout, TextOptions}; // requer [dependencies] speedy2d = "1.12.0"
use speedy2d::window::{WindowHandler, WindowHelper};
use speedy2d::{Graphics2D, Window};

mod comunicacao;
mod controlador;
mod transito;

use transito::{Transito, Via, VIAH_LARGURA, VIAV_LARGURA};
use transito::{VIAH_MARGEM, VIAH_TOTAL, VIAV_MARGEM, VIAV_TOTAL};
//use transito::veiculos::Carro;

use controlador::{Controle, TipoControlador};

use comunicacao::Comunicacao;

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

const TICKMS: f64 = 50.0; // Passo da simulação, em ms

// Struct necessária para a biblioteca gráfica
struct MyWindowHandler {
    simular: bool,        // true significa não está pausado
    finalizada: bool,     // true significa simulação concluida
    simulacao: Simulacao, // Dados da simulação
    largura_total: f64,   // Largura total da janela em pixels
    altura_total: f64,    // Altura total da janela em pixels
    fonte: Font,          // Fonte a ser usado na janela grafica
}

// Callbacks da biblioteca gráfica chegam nestes métodos
impl WindowHandler for MyWindowHandler {
    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        // Caso simulação tenha sido finalizada, não faz nada
        if self.finalizada {
            return;
        }

        // Executa um passo de simulação
        self.finalizada = !laco_simulacao(&mut self.simulacao);

        // Limpa a tela
        //graphics.clear_screen(Color::WHITE);
        graphics.clear_screen(Color::from_rgb(0.9, 0.9, 0.9));

        // Calcula resolução
        let resolucao_h = self.largura_total / VIAH_TOTAL; // relacao pixel por metro
        let resolucao_v = self.altura_total / VIAV_TOTAL; // relação pixel por metro
        let escala = (self.largura_total / 40.0) as f32;

        // Desenha linhas via H
        let y1 = (VIAV_MARGEM * resolucao_v) as f32;
        let y2 = ((VIAV_MARGEM + VIAH_LARGURA) * resolucao_v) as f32;
        graphics.draw_line(
            (0.0, y1),
            (self.largura_total as f32, y1),
            4.0,
            Color::BLACK,
        );
        graphics.draw_line(
            (0.0, y2),
            (self.largura_total as f32, y2),
            4.0,
            Color::BLACK,
        );

        // Desenha linhas via V
        let zero_h = (self.largura_total - (VIAH_MARGEM + VIAV_LARGURA) * resolucao_h) as f32;
        let x = (self.largura_total - (VIAH_MARGEM) * resolucao_h) as f32;
        graphics.draw_line(
            (zero_h, 0.0),
            (zero_h, self.altura_total as f32),
            4.0,
            Color::BLACK,
        );
        graphics.draw_line((x, 0.0), (x, self.altura_total as f32), 4.0, Color::BLACK);

        // Desenha os carros da Via H
        let mut lado = 0.0;
        for carro in self.simulacao.transito.get_iterador(Via::ViaH) {
            let x = zero_h + ((carro.pos_atual - carro.comprimento) * resolucao_h) as f32;
            let y = ((VIAV_MARGEM + 1.0) * resolucao_v) as f32;
            let w = (carro.comprimento * resolucao_h) as f32;
            //let h = (carro.largura * resolucao_v) as f32;
            let h = (2.0 * resolucao_v) as f32;

            let cor: Color;
            if carro.acel_atual == 0.0 {
                cor = Color::WHITE;
            } else if carro.acel_atual < 0.0 {
                cor = Color::RED;
            } else {
                cor = Color::GREEN;
            }

            let vertices = [
                Vector2 { x, y },
                Vector2 { x: x + w, y },
                Vector2 { x: x + w, y: y + h },
                Vector2 { x, y: y + h },
            ];
            graphics.draw_quad(vertices, cor);

            // Se foi pausada ou finalizada escreve na janela dados do carro
            if self.finalizada || !self.simular {
                if lado == 0.0 {
                    // Para espalhar texto na tela
                    lado = -80.0;
                } else {
                    lado = 0.0;
                }

                let tex_1 = carro.placa.to_string();
                let texto_1 = self.fonte.layout_text(&tex_1, escala, TextOptions::new());
                graphics.draw_text((x - 10.0, y + 10.0 + lado), Color::BLACK, &texto_1);

                let tex_2 = format!("A:{:.2}", carro.acel_atual);
                let texto_2 = self.fonte.layout_text(&tex_2, escala, TextOptions::new());
                graphics.draw_text((x - 10.0, y + 25.0 + lado), Color::BLACK, &texto_2);

                let tex_3 = format!("V:{:.2}", carro.vel_atual);
                let texto_3 = self.fonte.layout_text(&tex_3, escala, TextOptions::new());
                graphics.draw_text((x - 10.0, y + 40.0 + lado), Color::BLACK, &texto_3);

                let tex_4 = format!("P:{:.2}", carro.pos_atual);
                let texto_4 = self.fonte.layout_text(&tex_4, escala, TextOptions::new());
                graphics.draw_text((x - 10.0, y + 55.0 + lado), Color::BLACK, &texto_4);
            }
        }

        // Desenha os carros da Via V
        let mut lado = 0.0;
        for carro in self.simulacao.transito.get_iterador(Via::ViaV) {
            let x = (self.largura_total - (VIAH_MARGEM + VIAV_LARGURA - 1.0) * resolucao_h) as f32;
            let y = ((VIAV_MARGEM + VIAH_LARGURA - carro.pos_atual) * resolucao_v) as f32;
            let w = (2.0 * resolucao_h) as f32;
            let h = (carro.comprimento * resolucao_v) as f32;

            let cor: Color;
            if carro.acel_atual == 0.0 {
                cor = Color::WHITE;
            } else if carro.acel_atual < 0.0 {
                cor = Color::RED;
            } else {
                cor = Color::GREEN;
            }

            let vertices = [
                Vector2 { x, y },
                Vector2 { x: x + w, y },
                Vector2 { x: x + w, y: y + h },
                Vector2 { x, y: y + h },
            ];
            graphics.draw_quad(vertices, cor);

            // Se foi pausada ou finalizada escreve na janela dados do carro
            if self.finalizada || !self.simular {
                if lado == 0.0 {
                    // Para espalhar texto na tela
                    lado = 70.0;
                } else {
                    lado = 0.0;
                }

                let tex_1 = carro.placa.to_string();
                let texto_1 = self.fonte.layout_text(&tex_1, escala, TextOptions::new());
                graphics.draw_text((x - 50.0 + lado, y + 5.0), Color::BLACK, &texto_1);

                let tex_2 = format!("A:{:.2}", carro.acel_atual);
                let texto_2 = self.fonte.layout_text(&tex_2, escala, TextOptions::new());
                graphics.draw_text((x - 50.0 + lado, y + 20.0), Color::BLACK, &texto_2);

                let tex_3 = format!("V:{:.2}", carro.vel_atual);
                let texto_3 = self.fonte.layout_text(&tex_3, escala, TextOptions::new());
                graphics.draw_text((x - 50.0 + lado, y + 35.0), Color::BLACK, &texto_3);

                let tex_4 = format!("P:{:.2}", carro.pos_atual);
                let texto_4 = self.fonte.layout_text(&tex_4, escala, TextOptions::new());
                graphics.draw_text((x - 50.0 + lado, y + 50.0), Color::BLACK, &texto_4);
            }
        }

        // O que acontece agora ?
        if self.finalizada {
            println!("Simulação foi finalizada!");
            println!("Tecle algo na janela gráfica para terminar.");
        } else if self.simular {
            // Prossegue simulação
            helper.request_redraw();
        } else {
            println!("Simulaço foi pausada");
        }
    }

    fn on_keyboard_char(&mut self, helper: &mut WindowHelper<()>, tecla: char) {
        if tecla == 'x' {
            self.finalizada = true;
        }

        if self.finalizada {
            std::process::exit(1);
        } else {
            self.simular = !self.simular;
            if self.simular {
                // Voltando do pause
                helper.request_redraw();
            } else {
                // Entrando no pause
            }
        }
    }
}

// Sorteia tempo até a chegada do próximo veículo
fn tempo_entre_chegadas(min: f64, max: f64) -> f64 {
    rand::thread_rng().gen_range(min..=max)
}

// Descritor da simulação como um todo
struct Simulacao {
    cont: char,
    tec_min: f64,
    tec_max: f64,
    transito: Transito,
    comunicacao: Comunicacao,
    controle: Controle,
    tempo_ateh_proxima_chegada: f64,
}

// Laço de simulação, returna false no caso de finalizar a simulação
fn laco_simulacao(simul: &mut Simulacao) -> bool {
    // Mantém o tempo simulado próximo do tempo real, dorme TICKMS ms
    sleep(Duration::from_millis(TICKMS.round() as u64));

    // Atualiza estado do trânsito
    simul.transito.tick(TICKMS, &mut simul.comunicacao);

    // Atualiza estado do controlador
    simul.controle.acao_controle(TICKMS, &mut simul.comunicacao);

    // Mostra estado das vias
    simul.transito.mostra_vias();

    // Aborta a simulação se ocorreu colisão
    if let Some(m) = simul.transito.ocorreu_colisao() {
        println!(
            "Ocorreu colisao, controlador {}, tempos entre {} e {}: {}",
            simul.cont, simul.tec_min, simul.tec_max, m
        );
        return false;
    }

    // Verifica se tem algum carro no sistema
    if simul.transito.vazio() {
        println!("Nenhum carro no perímetro");
        return false;
    }

    // Verifica se está na hora de chegar novos carros
    simul.tempo_ateh_proxima_chegada -= TICKMS;

    if simul.tempo_ateh_proxima_chegada <= 0.0 {
        match simul
            .transito
            .chega_carro(Via::ViaH, &mut simul.comunicacao)
        {
            Ok(_) => (),
            Err(msg) => println!("Falha em chegar um carro via H: {}", msg),
        }

        match simul
            .transito
            .chega_carro(Via::ViaV, &mut simul.comunicacao)
        {
            Ok(_) => (),
            Err(msg) => println!("Falha em chegar um carro via V: {}", msg),
        }

        simul.tempo_ateh_proxima_chegada += tempo_entre_chegadas(simul.tec_min, simul.tec_max);
    }

    println!(
        "#main: tempo_ateh_proxima_chegada {}",
        simul.tempo_ateh_proxima_chegada
    );

    true
}

// Cria os principais componentes da simulação, cria a janela para visualização, aciona laço da biblioteca gráfica
fn simula_mundo(cont: char, tec_min: f64, tec_max: f64, tam_janela: f64) {
    // Cria um sistema de comunicação
    let mut comunicacao = Comunicacao::new();

    // Cria uma descrição de trânsito
    let mut transito = Transito::new();

    // Cria o primeiro carro da via H			!!!
    match transito.chega_carro(Via::ViaH, &mut comunicacao) {
        Ok(_) => (),
        Err(msg) => println!("Via H: {}", msg),
    };

    // Cria o primeiro carro da via V			!!!
    match transito.chega_carro(Via::ViaV, &mut comunicacao) {
        Ok(_) => (),
        Err(msg) => println!("Via V: {}", msg),
    };

    // Cria uma estrutura de controle
    let controle = match cont {
        's' => Controle::new(TipoControlador::Semaforo),
        'n' => Controle::new(TipoControlador::FazNada),
        _other => panic!("Tipo de controlador não é s|n."),
    };

    // Descritor da simulação
    let simul = Simulacao {
        cont,
        tec_min, // Tempo entre chegadas
        tec_max,
        transito,
        comunicacao,
        controle,
        tempo_ateh_proxima_chegada: tempo_entre_chegadas(tec_min, tec_max),
    };

    // Cria janela sem evento de usuario
    let window =
        Window::new_centered("Cruzamento", (tam_janela as u32, tam_janela as u32)).unwrap();
    // Cria fonte a ser usado na janela grafica
    let fonte = Font::new(include_bytes!("../assets/fonts/NotoSans-Regular.ttf")).unwrap();

    // Laço ficará com o WindowHandler
    println!("simula_transito");
    window.run_loop(MyWindowHandler {
        simular: true,
        finalizada: false,
        simulacao: simul,
        altura_total: tam_janela,
        largura_total: tam_janela,
        fonte,
    });
}

// Confere os argumentos da linha de comando e chama 'simula_mundo'
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        panic!("Uso: <s|n>  <min entre chegadas>  <max entre chegadas> <tam janela>");
    }

    let cont = args[1]
        .trim()
        .chars()
        .next()
        .expect("Uso: <s|n>  <min entre chegadas>  <max entre chegadas> <tam janela>");

    match cont {
        's' | 'n' => (),
        _other => panic!("Uso: <s|n>  <min entre chegadas>  <max entre chegadas> <tam janela>"),
    };

    let result_tec_min = args[2].trim().parse::<f64>();
    let tec_min = result_tec_min
        .expect("Uso: <s|n>  <min entre chegadas>  <max entre chegadas> <tam janela>");

    let result_tec_max = args[3].trim().parse::<f64>();
    let tec_max = result_tec_max
        .expect("Uso: <s|n>  <min entre chegadas>  <max entre chegadas> <tam janela>");

    if tec_min < 2.0 || tec_max < 2.0 {
        println!("Tempo entre chegadas deve ser no mínimo 2 segundos.");
        return;
    }

    if tec_min > tec_max {
        println!(
            "Tempo entre chegadas mínimo não pode ser maior que o tempo entre chegadas máximo."
        );
        return;
    }

    let result_tam_janela = args[4].trim().parse::<f64>();
    let tam_janela = result_tam_janela
        .expect("Uso: <s|n>  <min entre chegadas>  <max entre chegadas> <tam janela>");
    if !(200.0..=1000.0).contains(&tam_janela) {
        println!("Tamanho da janela deve estar entre 200 e 1000.");
        return;
    }

    println!("Inicio da simulação de cruzamento automático");

    simula_mundo(cont, 1000.0 * tec_min, 1000.0 * tec_max, tam_janela);

    println!("Fim da simulação de cruzamento automático");
}
