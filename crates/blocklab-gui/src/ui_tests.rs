//! Testes da interface rodando de verdade (sem janela): o egui_kittest
//! executa os quadros e simula mouse e teclado.

use std::time::Duration;

use blocklab_core::algebra::RationalFn;
use blocklab_core::diagram::{Diagram, NodeKind, Pos, Sign, examples};
use blocklab_core::format::Mode;
use blocklab_core::parser::parse_rational;
use eframe::egui::{
    Event, Key, Modifiers, PointerButton, Pos2, Theme, ThemePreference, TouchDeviceId, TouchId,
    TouchPhase, Vec2, pos2, vec2,
};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use crate::app::App;
use crate::geometry;
use crate::home::{Page, Screen};
use crate::layout::Layout;

fn harness(d: Diagram) -> Harness<'static, App> {
    harness_sized(d, vec2(1280.0, 780.0))
}

/// Janela do tamanho dos protótipos: 390 × 844 (celular), 1194 × 834 (tablet).
fn harness_sized(d: Diagram, size: Vec2) -> Harness<'static, App> {
    let mut h = Harness::builder()
        .with_size(size)
        // Quadros de 1/60 s, como numa tela real (o padrão de 1/4 s deixa
        // dois cliques seguidos longe demais para virar duplo clique).
        .with_step_dt(1.0 / 60.0)
        .build_eframe(move |cc| App::new(&cc.egui_ctx, d, None, Mode::Symbolic));
    h.run_steps(3); // primeiro quadro enquadra o diagrama
    h
}

/// Espera o cálculo em segundo plano terminar.
fn settle(h: &mut Harness<'static, App>) {
    for _ in 0..1000 {
        h.run_steps(1);
        let s = h.state();
        if !s.solver.is_running() && s.result.has_outcome() {
            h.run_steps(2);
            return;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("o cálculo não terminou");
}

fn transfer(h: &Harness<'static, App>) -> RationalFn {
    match h.state().result.outcome() {
        Some(Ok(r)) => r.transfer.clone(),
        other => panic!("sem resultado: {other:?}"),
    }
}

fn screen(h: &Harness<'static, App>, world: Pos2) -> Pos2 {
    h.state().view.to_screen(world)
}

fn click(h: &mut Harness<'static, App>, p: Pos2) {
    h.hover_at(p);
    h.run_steps(1);
    h.drag_at(p);
    h.run_steps(1);
    h.drop_at(p);
    h.run_steps(2);
}

/// Pressiona em `a`, arrasta em passos até `b` e solta.
fn drag(h: &mut Harness<'static, App>, a: Pos2, b: Pos2) {
    h.hover_at(a);
    h.run_steps(1);
    h.drag_at(a);
    h.run_steps(1);
    for k in 1..=8 {
        h.hover_at(a + (b - a) * (k as f32 / 8.0));
        h.run_steps(1);
    }
    h.drop_at(b);
    h.run_steps(2);
}

fn node_named(h: &Harness<'static, App>, expr: &str) -> blocklab_core::diagram::Node {
    h.state()
        .doc
        .diagram
        .nodes
        .iter()
        .find(|n| matches!(&n.kind, NodeKind::Block { expr: e } if e == expr))
        .cloned()
        .unwrap()
}

fn reference(sign: &str) -> RationalFn {
    parse_rational(&format!(
        "G1 G4 (G2+G3) / (1 {sign} G1 G4 H1 + G1 G4 (G2+G3) H2)"
    ))
    .unwrap()
}

#[test]
fn abre_e_resolve_a_referencia() {
    let mut h = harness(examples::reference(Sign::Plus));
    settle(&mut h);
    assert_eq!(transfer(&h), reference("-"));
}

#[test]
fn clique_no_sinal_alterna_e_desfazer_volta() {
    let mut h = harness(examples::reference(Sign::Plus));
    settle(&mut h);
    // Σ2 é o segundo somador; a entrada 1 recebe H1.
    let sum2 = h
        .state()
        .doc
        .diagram
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Sum { .. }))
        .nth(1)
        .cloned()
        .unwrap();
    let at = screen(&h, geometry::sign_pos(&sum2, 1));
    click(&mut h, at);
    settle(&mut h);
    assert_eq!(transfer(&h), reference("+"));

    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run_steps(2);
    settle(&mut h);
    assert_eq!(transfer(&h), reference("-"));
}

#[test]
fn ligar_fio_arrastando_de_porta_a_porta() {
    let mut d = Diagram::new();
    let x = d.add_input("X", Pos::new(0.0, 0.0));
    let y = d.add_output("Y", Pos::new(300.0, 0.0));
    let mut h = harness(d);
    let (nx, ny) = {
        let d = &h.state().doc.diagram;
        (d.node(x).cloned().unwrap(), d.node(y).cloned().unwrap())
    };
    let a = screen(&h, geometry::out_port(&nx, 0).pos);
    let b = screen(&h, geometry::in_port(&ny, 0).pos);
    drag(&mut h, a, b);
    assert_eq!(h.state().doc.diagram.wires.len(), 1);
    settle(&mut h);
    assert!(transfer(&h).is_one());
}

#[test]
fn apagar_com_delete_e_desfazer() {
    let mut h = harness(examples::series());
    let g1 = node_named(&h, "G1");
    let at = screen(&h, geometry::center(&g1));
    click(&mut h, at);
    assert!(h.state().sel.nodes.contains(&g1.id));
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert!(h.state().doc.diagram.node(g1.id).is_none());
    // O fio ligado ao nó também some.
    assert_eq!(h.state().doc.diagram.wires.len(), 1);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run_steps(2);
    assert!(h.state().doc.diagram.node(g1.id).is_some());
    assert_eq!(h.state().doc.diagram.wires.len(), 3);
}

#[test]
fn duplo_clique_edita_a_expressao() {
    let mut h = harness(examples::series());
    let g1 = node_named(&h, "G1");
    let at = screen(&h, geometry::center(&g1));
    click(&mut h, at);
    click(&mut h, at);
    let editing = h.state().editing.as_ref().map(|e| e.node);
    assert_eq!(editing, Some(g1.id));
    h.state_mut().editing.as_mut().unwrap().text = "K/(s+1)".into();
    h.run_steps(1);
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert!(h.state().editing.is_none());
    assert_eq!(
        h.state().doc.diagram.node(g1.id).unwrap().kind,
        NodeKind::Block {
            expr: "K/(s+1)".into()
        }
    );
    settle(&mut h);
    assert_eq!(transfer(&h), parse_rational("K G2 / (s + 1)").unwrap());
}

#[test]
fn mover_no_encaixa_na_grade_em_um_passo_de_historico() {
    let mut h = harness(examples::series());
    let g1 = node_named(&h, "G1");
    let at = screen(&h, geometry::center(&g1));
    drag(&mut h, at, at + vec2(57.0, 23.0));
    let moved = node_named(&h, "G1");
    assert_ne!(moved.pos, g1.pos);
    assert_eq!(moved.pos.x % geometry::GRID, 0.0);
    assert_eq!(moved.pos.y % geometry::GRID, 0.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run_steps(2);
    assert_eq!(node_named(&h, "G1").pos, g1.pos);
}

#[test]
fn retangulo_de_selecao() {
    let mut h = harness(examples::series());
    // Do canto superior esquerdo do canvas até o meio dele: pega X e G1.
    let canvas = h.state().canvas_rect;
    let x = h
        .state()
        .doc
        .diagram
        .nodes
        .iter()
        .find(|n| matches!(n.kind, NodeKind::Input { .. }))
        .cloned()
        .unwrap();
    let g1 = node_named(&h, "G1");
    let right_of_g1 = screen(&h, geometry::rect(&g1).right_bottom()) + vec2(5.0, 5.0);
    drag(
        &mut h,
        canvas.min + vec2(5.0, 5.0),
        pos2(right_of_g1.x, canvas.bottom() - 5.0),
    );
    let sel = &h.state().sel.nodes;
    assert!(sel.contains(&x.id) && sel.contains(&g1.id), "{sel:?}");
    assert_eq!(sel.len(), 2);
}

#[test]
fn so_olhar_nao_marca_o_documento_como_alterado() {
    // O painel mostra os campos de parâmetro; exibi-los não pode gravar
    // valores vazios no diagrama (senão todo arquivo aberto pediria
    // "salvar alterações?" ao fechar).
    let mut h = harness(examples::reference(Sign::Plus));
    settle(&mut h);
    h.run_steps(5);
    assert!(h.state().doc.diagram.params.is_empty());
    assert!(!h.state().is_dirty());
    assert_eq!(
        h.state().title,
        "sem título — controLAB — Diagrama de Blocos"
    );
}

#[test]
fn tela_inicial_abre_exemplo_e_diagrama_novo() {
    let mut h = harness(examples::series());
    h.state_mut().show_home();
    h.run_steps(2);
    assert_eq!(h.state().screen, Screen::Home);

    // Escolhe "Laços aninhados" na grade e abre pelo botão da prévia.
    h.get_by_label("Laços aninhados").click();
    h.run_steps(2);
    h.get_by_label("Abrir exemplo no editor").click();
    h.run_steps(2);
    assert_eq!(h.state().screen, Screen::Editor);
    assert_eq!(h.state().doc.diagram, examples::nested_loops());

    h.state_mut().screen = Screen::Home;
    h.run_steps(2);
    h.get_by_label("Novo diagrama").click();
    h.run_steps(2);
    assert_eq!(h.state().screen, Screen::Editor);
    assert!(h.state().doc.diagram.nodes.is_empty());
}

#[test]
fn menu_visualizacao_troca_o_tema() {
    let mut h = harness(examples::series());
    h.get_by_label("Visualização").click();
    h.run_steps(2);
    h.get_by_label("Modo escuro").click();
    h.run_steps(2);
    assert_eq!(h.state().prefs.theme, ThemePreference::Dark);
    assert_eq!(h.ctx.theme(), Theme::Dark);
    h.get_by_label("Visualização").click();
    h.run_steps(2);
    h.get_by_label("Modo claro").click();
    h.run_steps(2);
    assert_eq!(h.ctx.theme(), Theme::Light);
}

#[test]
fn passar_o_mouse_no_texto_de_um_laco_destaca_no_diagrama() {
    let mut h = harness(examples::reference(Sign::Plus));
    settle(&mut h);
    // Bem em cima do texto da rota (antes, o texto "cobria" o hover).
    let route = "Σ2 → G1 → G4 → H1 → Σ2";
    h.get_by_label(route).scroll_to_me();
    h.run_steps(3);
    h.get_by_label(route).hover();
    h.run_steps(2);
    let lit = h.state().highlight.clone();
    let names: Vec<String> = lit
        .iter()
        .filter_map(|id| h.state().doc.diagram.node(*id))
        .filter_map(|n| match &n.kind {
            NodeKind::Block { expr } => Some(expr.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(lit.len(), 5, "{names:?}");
    for b in ["G1", "G4", "H1"] {
        assert!(names.iter().any(|n| n == b), "{names:?}");
    }
}

// --- Tablet e celular ------------------------------------------------------

const PHONE: Vec2 = vec2(390.0, 844.0);
const TABLET: Vec2 = vec2(1194.0, 834.0);

#[test]
fn celular_tela_inicial_previa_e_abrir_exemplo() {
    let mut h = harness_sized(examples::series(), PHONE);
    h.state_mut().show_home();
    h.run_steps(2);
    assert_eq!(h.state().layout, Layout::Phone);

    // Barra inferior troca de página.
    h.get_by_label("Exemplos").click();
    h.run_steps(2);
    assert_eq!(h.state().home.page, Page::Examples);
    h.get_by_label("Início").click();
    h.run_steps(2);

    // Tocar num exemplo abre a prévia; o botão dela abre no editor.
    h.get_by_label("Laços aninhados").click();
    h.run_steps(3);
    assert!(h.state().home.sheet);
    h.get_by_label("Abrir no editor").click();
    h.run_steps(3);
    assert_eq!(h.state().screen, Screen::Editor);
    assert_eq!(h.state().doc.diagram, examples::nested_loops());

    // E o "voltar" da barra superior leva à tela inicial.
    h.get_by_label("Tela inicial").click();
    h.run_steps(2);
    assert_eq!(h.state().screen, Screen::Home);
    h.get_by_label("Novo diagrama").click();
    h.run_steps(2);
    assert_eq!(h.state().screen, Screen::Editor);
    assert!(h.state().doc.diagram.nodes.is_empty());
}

#[test]
fn tablet_trilho_navega_e_previa_fica_ao_lado() {
    let mut h = harness_sized(examples::series(), TABLET);
    h.state_mut().show_home();
    h.run_steps(2);
    assert_eq!(h.state().layout, Layout::Tablet);
    h.get_by_label("Aprender").click();
    h.run_steps(2);
    assert_eq!(h.state().home.page, Page::Learn);
    h.get_by_label("Exemplos").click();
    h.run_steps(2);
    h.get_by_label("Blocos em paralelo").click();
    h.run_steps(2);
    // Com 1194 px a prévia é um painel, não uma janela por cima.
    assert!(!h.state().home.sheet);
    h.get_by_label("Abrir no editor").click();
    h.run_steps(3);
    assert_eq!(h.state().doc.diagram, examples::parallel());
}

#[test]
fn celular_tocar_na_paleta_cria_sem_empilhar() {
    let mut h = harness_sized(Diagram::new(), PHONE);
    h.get_by_label("Bloco").click();
    h.run_steps(2);
    h.get_by_label("Bloco").click();
    h.run_steps(2);
    let d = &h.state().doc.diagram;
    assert_eq!(d.nodes.len(), 2);
    assert_ne!(d.nodes[0].pos, d.nodes[1].pos);
}

#[test]
fn celular_barra_de_acoes_apaga_e_desfazer_volta() {
    let mut h = harness_sized(examples::series(), PHONE);
    let g1 = node_named(&h, "G1");
    let at = screen(&h, geometry::center(&g1));
    click(&mut h, at);
    assert!(h.state().sel.nodes.contains(&g1.id));
    h.get_by_label("Apagar").click();
    h.run_steps(2);
    assert!(h.state().doc.diagram.node(g1.id).is_none());
    h.get_by_label("Desfazer").click();
    h.run_steps(2);
    assert!(h.state().doc.diagram.node(g1.id).is_some());
}

#[test]
fn celular_folha_de_resultado_abre_e_fecha() {
    let mut h = harness_sized(examples::reference(Sign::Plus), PHONE);
    settle(&mut h);
    h.get_by_label("Ver resultado").click();
    h.run_steps(2);
    assert!(h.state().result_sheet);
    h.get_by_label("Recolher resultado").click();
    h.run_steps(2);
    assert!(!h.state().result_sheet);
}

#[test]
fn tablet_arrastar_no_vazio_move_a_vista() {
    let mut h = harness_sized(examples::series(), TABLET);
    let before = h.state().view.offset;
    let canvas = h.state().canvas_rect;
    let a = canvas.center() + vec2(0.0, canvas.height() * 0.35);
    drag(&mut h, a, a + vec2(80.0, -40.0));
    let moved = h.state().view.offset - before;
    assert!((moved - vec2(80.0, -40.0)).length() < 1.0, "{moved:?}");
    assert!(h.state().sel.is_empty());
}

#[test]
fn tablet_tocar_num_laco_fixa_o_destaque() {
    let mut h = harness_sized(examples::reference(Sign::Plus), TABLET);
    settle(&mut h);
    let route = "Σ2 → G1 → G4 → H1 → Σ2";
    h.get_by_label(route).scroll_to_me();
    h.run_steps(3);
    h.get_by_label(route).click();
    h.run_steps(2);
    // O ponteiro sai de cima, mas o laço continua destacado.
    h.hover_at(h.state().canvas_rect.center());
    h.run_steps(2);
    assert!(h.state().pinned.is_some());
    assert_eq!(h.state().highlight.len(), 5);
    h.get_by_label(route).click();
    h.run_steps(2);
    assert!(h.state().pinned.is_none());
}

/// Dedo na tela como o egui-winit entrega: o evento de toque e, para o
/// primeiro dedo, o ponteiro emulado.
fn finger(h: &Harness<'static, App>, id: u64, phase: TouchPhase, pos: Pos2) {
    h.event(Event::Touch {
        device_id: TouchDeviceId(1),
        id: TouchId(id),
        phase,
        pos,
        force: None,
    });
    if id != 0 {
        return;
    }
    match phase {
        TouchPhase::Start => {
            h.event(Event::PointerMoved(pos));
            h.event(Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            });
        }
        TouchPhase::Move => h.event(Event::PointerMoved(pos)),
        TouchPhase::End | TouchPhase::Cancel => {
            h.event(Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            });
            h.event(Event::PointerGone);
        }
    }
}

#[test]
fn pinca_da_zoom_e_cancela_o_arrasto_do_primeiro_dedo() {
    let mut h = harness_sized(examples::series(), TABLET);
    let before = h.state().doc.diagram.clone();
    let zoom = h.state().view.zoom;
    let g1 = node_named(&h, "G1");
    let a = screen(&h, geometry::center(&g1));
    let b = a + vec2(0.0, 120.0);

    // O primeiro dedo pega o bloco e começa a arrastá-lo...
    finger(&h, 0, TouchPhase::Start, a);
    h.run_steps(1);
    for k in 1..=3 {
        finger(&h, 0, TouchPhase::Move, a + vec2(8.0 * k as f32, 0.0));
        h.run_steps(1);
    }
    let a = a + vec2(24.0, 0.0);
    // ...e o segundo dedo transforma o gesto em pinça.
    finger(&h, 1, TouchPhase::Start, b);
    h.run_steps(1);
    for k in 1..=6 {
        let d = 12.0 * k as f32;
        finger(&h, 0, TouchPhase::Move, a - vec2(0.0, d));
        finger(&h, 1, TouchPhase::Move, b + vec2(0.0, d));
        h.run_steps(1);
    }
    finger(&h, 1, TouchPhase::End, b + vec2(0.0, 72.0));
    h.run_steps(1);
    finger(&h, 0, TouchPhase::End, a - vec2(0.0, 72.0));
    h.run_steps(3);

    assert!(h.state().view.zoom > zoom * 1.5, "{}", h.state().view.zoom);
    // O bloco voltou ao lugar e nada entrou no histórico.
    assert_eq!(h.state().doc.diagram, before);
    assert!(!h.state().doc.can_undo());
    assert!(!h.state().touch_lock);
}

/// Recuos como os de um celular com navegação por gestos, em retrato:
/// barra de status de 48 pt em cima, 24 pt de gestos embaixo, recorte
/// lateral de 12 pt (como em paisagem) para conferir as laterais.
fn insets() -> eframe::egui::SafeAreaInsets {
    eframe::egui::SafeAreaInsets(eframe::egui::epaint::MarginF32 {
        left: 12.0,
        right: 12.0,
        top: 48.0,
        bottom: 24.0,
    })
}

/// Nada que se toca pode ficar nas faixas cobertas pelo sistema.
fn assert_inside_safe_area(h: &Harness<'static, App>, labels: &[&str]) {
    let safe = h.ctx.content_rect();
    for label in labels {
        let r = h.get_by_label(label).rect();
        assert!(
            safe.expand(0.5).contains_rect(r),
            "{label}: {r:?} fora de {safe:?}"
        );
    }
}

#[test]
fn area_segura_afasta_a_interface_das_barras_do_sistema() {
    let mut h = harness_sized(examples::reference(Sign::Plus), PHONE);
    h.input_mut().safe_area_insets = Some(insets());
    h.run_steps(3);
    let safe = h.ctx.content_rect();
    assert_eq!(
        safe,
        eframe::egui::Rect::from_min_max(pos2(12.0, 48.0), pos2(378.0, 820.0))
    );

    // Editor: barra superior, paleta, faixa do resultado e zoom.
    assert_inside_safe_area(
        &h,
        &[
            "Tela inicial",
            "Mais opções",
            "Desfazer",
            "Bloco",
            "Saída",
            "Ver resultado",
            "Aumentar zoom",
            "Seleção múltipla",
        ],
    );
    assert!(h.state().canvas_rect.top() >= safe.top());
    assert!(h.state().canvas_rect.right() <= safe.right());

    // Tela inicial: barra superior e navegação de baixo.
    h.state_mut().show_home();
    h.run_steps(3);
    assert_inside_safe_area(&h, &["Novo diagrama", "Mais opções", "Início", "Aprender"]);

    // Sem recuos (computador), a interface volta a ocupar a janela toda.
    h.input_mut().safe_area_insets = Some(Default::default());
    h.run_steps(3);
    assert_eq!(h.ctx.content_rect(), h.ctx.viewport_rect());
}

#[test]
fn teclado_aberto_traz_o_bloco_em_edicao_para_cima() {
    let mut h = harness_sized(examples::series(), PHONE);
    let g1 = node_named(&h, "G1");
    // O teclado cobre do meio da tela para baixo.
    h.input_mut().safe_area_insets = Some(eframe::egui::SafeAreaInsets(
        eframe::egui::epaint::MarginF32 {
            bottom: 330.0,
            ..Default::default()
        },
    ));
    h.state_mut().view.offset.y += 200.0; // bloco bem embaixo
    h.state_mut().start_editing(g1.id);
    h.run_steps(3);
    let canvas = h.state().canvas_rect;
    let r = h.state().view.rect_to_screen(geometry::rect(&g1));
    assert!(canvas.contains_rect(r), "{r:?} fora de {canvas:?}");
}

#[test]
fn realimentacao_da_paleta_ja_vem_espelhada() {
    let mut h = harness(Diagram::new());
    h.get_by_label("Realimentação").click();
    h.run_steps(2);
    let n = h.state().doc.diagram.nodes[0].clone();
    assert!(n.flipped);
    assert_eq!(n.kind, NodeKind::Block { expr: "H1".into() });
    // Entrada à direita e saída à esquerda, como no ramo de retorno.
    assert!(geometry::in_port(&n, 0).pos.x > geometry::out_port(&n, 0).pos.x);
}

#[test]
fn malha_com_realimentacao_da_paleta_sem_espelhar() {
    // A malha de realimentação sem o H: X → Σ(+, −) → G → • → Y.
    let mut d = examples::feedback(Sign::Minus);
    let old_h = d
        .nodes
        .iter()
        .find(|n| matches!(&n.kind, NodeKind::Block { expr } if expr == "H"))
        .map(|n| n.id)
        .unwrap();
    d.remove_node(old_h);
    crate::doc::compact_branches(&mut d);
    let mut h = harness(d);
    h.get_by_label("Realimentação").click();
    h.run_steps(2);
    let hb = node_named(&h, "H1");
    assert!(hb.flipped);

    // Ramificação → entrada do H (à direita) e saída do H (à esquerda) →
    // entrada de baixo do somador, só arrastando fios.
    let (branch_out, sum_in) = {
        let d = &h.state().doc.diagram;
        let hs = geometry::handles(d);
        let branch_out = hs
            .iter()
            .find(|x| {
                x.dir == blocklab_core::diagram::PortDir::Out
                    && matches!(d.node(x.port.node).unwrap().kind, NodeKind::Branch { .. })
            })
            .unwrap()
            .pos;
        let sum = d
            .nodes
            .iter()
            .find(|n| matches!(n.kind, NodeKind::Sum { .. }))
            .unwrap();
        (branch_out, geometry::in_port(sum, 1).pos)
    };
    let a = screen(&h, branch_out);
    let b = screen(&h, geometry::in_port(&hb, 0).pos);
    drag(&mut h, a, b);
    let a = screen(&h, geometry::out_port(&hb, 0).pos);
    let b = screen(&h, sum_in);
    drag(&mut h, a, b);

    settle(&mut h);
    assert_eq!(transfer(&h), parse_rational("G / (1 + G H1)").unwrap());
}
