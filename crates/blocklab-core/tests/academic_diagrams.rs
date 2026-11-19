//! Block diagram validation for UFERSA Control Systems II academic lists
//! (`Exercicios_-_LGR.pdf` Question 3 and `Projeto_de_controladores.pdf` Questions 3, 4, 6, 7, 8, 9)
//! using `blocklab_core::solve` (Mason's gain formula over exact rational functions).

use blocklab_core::{
    algebra::RationalFn,
    diagram::{Diagram, NodeId, PortRef, Pos, Side, Sign, examples},
    parser::parse_rational,
    solve,
};

fn p(x: f32, y: f32) -> Pos {
    Pos::new(x, y)
}

fn expr(src: &str) -> RationalFn {
    parse_rational(src).unwrap_or_else(|e| panic!("failed to parse '{src}': {e}"))
}

fn link(d: &mut Diagram, a: NodeId, b: NodeId, port: u8) {
    d.link(a, b, port).unwrap();
}

fn link_out(d: &mut Diagram, a: NodeId, out: u8, b: NodeId, port: u8) {
    d.connect(PortRef::new(a, out), PortRef::new(b, port))
        .unwrap();
}

fn flip(d: &mut Diagram, id: NodeId) {
    d.node_mut(id).unwrap().flipped = true;
}

/// Question 3 of `Projeto_de_controladores.pdf`:
/// R(s) -> Sum1(+, -) -> G_c(s) -> Sum2(+, -) -> 18/(2s+11) -> 1/(2s+1) -> branch1 -> 1/(30s) -> branch2 -> C(s)
/// Inner feedback: branch1 -> 0.5 -> Sum2(-)
/// Outer feedback: branch2 -> 1 -> Sum1(-)
fn build_q3_servo_diagram(gc_expr: &str) -> Diagram {
    let mut d = Diagram::new();
    let r = d.add_input("R", p(40.0, 200.0));
    let sum1 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(120.0, 200.0),
    );
    let gc = d.add_block(gc_expr, p(220.0, 200.0));
    let sum2 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(320.0, 200.0),
    );
    let g_amp = d.add_block("18/(2*s + 11)", p(420.0, 200.0));
    let g_mot = d.add_block("1/(2*s + 1)", p(540.0, 200.0));
    let br_inner = d.add_branch(2, p(640.0, 200.0));
    let g_gear = d.add_block("1/(30*s)", p(740.0, 200.0));
    let br_outer = d.add_branch(2, p(840.0, 200.0));
    let c = d.add_output("C", p(940.0, 200.0));

    let h_inner = d.add_block("0.5", p(480.0, 300.0));
    let h_outer = d.add_block("1", p(480.0, 380.0));
    flip(&mut d, h_inner);
    flip(&mut d, h_outer);

    link(&mut d, r, sum1, 0);
    link(&mut d, sum1, gc, 0);
    link(&mut d, gc, sum2, 0);
    link(&mut d, sum2, g_amp, 0);
    link(&mut d, g_amp, g_mot, 0);
    link(&mut d, g_mot, br_inner, 0);
    link_out(&mut d, br_inner, 0, g_gear, 0);
    link_out(&mut d, br_inner, 1, h_inner, 0);
    link(&mut d, h_inner, sum2, 1);
    link(&mut d, g_gear, br_outer, 0);
    link_out(&mut d, br_outer, 0, c, 0);
    link_out(&mut d, br_outer, 1, h_outer, 0);
    link(&mut d, h_outer, sum1, 1);
    d
}

/// Question 7 of `Projeto_de_controladores.pdf`:
/// R(s) -> Sum1(+, -) -> G_c(s) -> 5/(s(s+2)) -> Sum2(+, -) -> 1/s -> br_in -> br_out -> C(s)
/// Inner feedback: br_in -> 2 -> Sum2(-)
/// Outer feedback: br_out -> 1 -> Sum1(-)
fn build_q7_cascade_diagram(gc_expr: &str) -> Diagram {
    let mut d = Diagram::new();
    let r = d.add_input("R", p(40.0, 200.0));
    let sum1 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(120.0, 200.0),
    );
    let gc = d.add_block(gc_expr, p(220.0, 200.0));
    let g1 = d.add_block("5/(s*(s + 2))", p(340.0, 200.0));
    let sum2 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(440.0, 200.0),
    );
    let g2 = d.add_block("1/s", p(540.0, 200.0));
    let br_in = d.add_branch(2, p(640.0, 200.0));
    let br_out = d.add_branch(2, p(720.0, 200.0));
    let c = d.add_output("C", p(820.0, 200.0));

    let h_in = d.add_block("2", p(540.0, 300.0));
    let h_out = d.add_block("1", p(440.0, 380.0));
    flip(&mut d, h_in);
    flip(&mut d, h_out);

    link(&mut d, r, sum1, 0);
    link(&mut d, sum1, gc, 0);
    link(&mut d, gc, g1, 0);
    link(&mut d, g1, sum2, 0);
    link(&mut d, sum2, g2, 0);
    link(&mut d, g2, br_in, 0);
    link_out(&mut d, br_in, 0, br_out, 0);
    link_out(&mut d, br_in, 1, h_in, 0);
    link(&mut d, h_in, sum2, 1);
    link_out(&mut d, br_out, 0, c, 0);
    link_out(&mut d, br_out, 1, h_out, 0);
    link(&mut d, h_out, sum1, 1);
    d
}

/// Question 9 of `Projeto_de_controladores.pdf`:
/// Second-order canonical block diagram with wn = 1, xi = 0 (2*xi*wn = 0):
/// R(s) -> Sum1(+, -) -> G_c(s) -> wn^2 -> Sum2(+, -) -> 1/s -> br_in -> 1/s -> br_out -> Y(s)
/// Inner feedback: br_in -> 2*xi*wn -> Sum2(-)
/// Outer feedback: br_out -> 1 -> Sum1(-)
fn build_q9_second_order_diagram(gc_expr: &str, wn_sq_expr: &str, two_xi_wn_expr: &str) -> Diagram {
    let mut d = Diagram::new();
    let r = d.add_input("R", p(40.0, 200.0));
    let sum1 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(120.0, 200.0),
    );
    let gc = d.add_block(gc_expr, p(220.0, 200.0));
    let wn2 = d.add_block(wn_sq_expr, p(340.0, 200.0));
    let sum2 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(440.0, 200.0),
    );
    let int1 = d.add_block("1/s", p(540.0, 200.0));
    let br_in = d.add_branch(2, p(640.0, 200.0));
    let int2 = d.add_block("1/s", p(740.0, 200.0));
    let br_out = d.add_branch(2, p(840.0, 200.0));
    let y = d.add_output("Y", p(940.0, 200.0));

    let h_in = d.add_block(two_xi_wn_expr, p(540.0, 300.0));
    let h_out = d.add_block("1", p(480.0, 380.0));
    flip(&mut d, h_in);
    flip(&mut d, h_out);

    link(&mut d, r, sum1, 0);
    link(&mut d, sum1, gc, 0);
    link(&mut d, gc, wn2, 0);
    link(&mut d, wn2, sum2, 0);
    link(&mut d, sum2, int1, 0);
    link(&mut d, int1, br_in, 0);
    link_out(&mut d, br_in, 0, int2, 0);
    link_out(&mut d, br_in, 1, h_in, 0);
    link(&mut d, h_in, sum2, 1);
    link(&mut d, int2, br_out, 0);
    link_out(&mut d, br_out, 0, y, 0);
    link_out(&mut d, br_out, 1, h_out, 0);
    link(&mut d, h_out, sum1, 1);
    d
}

#[test]
fn validate_lgr_question_3_block_diagrams_via_mason() {
    // Q3(a) Eq.(14): G(s) = K/(s(s^2+4s+5)), H(s) = 1
    let d3a = examples::feedback_with("K/(s*(s^2 + 4*s + 5))", "1", Sign::Minus);
    let r3a = solve(&d3a).unwrap();
    assert_eq!(r3a.transfer, expr("K/(s^3 + 4*s^2 + 5*s + K)"));

    // Q3(b) Eq.(15): G(s) = K/(s(s^2+4s+13)), H(s) = 1/(s+1)
    let d3b = examples::feedback_with("K/(s*(s^2 + 4*s + 13))", "1/(s + 1)", Sign::Minus);
    let r3b = solve(&d3b).unwrap();
    assert_eq!(
        r3b.transfer,
        expr("K*(s + 1)/(s^4 + 5*s^3 + 17*s^2 + 13*s + K)")
    );

    // Q3(c) Eq.(16): G(s) = K/(s+3.6), H(s) = (s+1)/s^2
    let d3c = examples::feedback_with("K/(s + 3.6)", "(s + 1)/(s^2)", Sign::Minus);
    let r3c = solve(&d3c).unwrap();
    assert_eq!(r3c.transfer, expr("K*s^2/(s^3 + 3.6*s^2 + K*s + K)"));
}

#[test]
fn validate_controller_design_block_diagrams_q3_q4_q6_q7_q8_q9_via_mason() {
    // 1. Q3 Servo Diagram:
    //    Uncompensated symbolic controller Gc:
    let d_q3_sym = build_q3_servo_diagram("Gc");
    let r_q3_sym = solve(&d_q3_sym).unwrap();
    assert_eq!(r_q3_sym.loops.len(), 2);
    assert_eq!(
        r_q3_sym.transfer,
        expr("(3*Gc)/(20*s^3 + 120*s^2 + 100*s + 3*Gc)")
    );
    //    Compensated with PD controller Gc(s) = (160 + 100*s)/3:
    let d_q3_pd = build_q3_servo_diagram("(160 + 100*s)/3");
    let r_q3_pd = solve(&d_q3_pd).unwrap();
    assert_eq!(r_q3_pd.transfer, expr("(5*s + 8)/(s^3 + 6*s^2 + 10*s + 8)"));
    assert_eq!(Diagram::from_json(&d_q3_pd.to_json()).unwrap(), d_q3_pd);

    // 2. Q4 Non-unity feedback diagram (G(s) = 5/(s+0.1), H(s) = 0.05, Gc(s) = (7.6s+16)/s):
    let d_q4 = examples::feedback_with("((7.6*s + 16)/s)*(5/(s + 0.1))", "0.05", Sign::Minus);
    let r_q4 = solve(&d_q4).unwrap();
    assert_eq!(r_q4.transfer, expr("(38*s + 80)/(s^2 + 2*s + 4)"));

    // 3. Q6 PD controller with exact pole-zero cancellation of (s + 2):
    //    G_c(s) = 8 + 4*s, G(s) = 1/(s*(s+2)^2), H(s) = 1
    //    Notice `blocklab-core` performs exact polynomial GCD cancellation, automatically
    //    reducing 4(s+2)/((s+2)(s^2+2s+4)) to the minimal realization 4/(s^2+2s+4)!
    let d_q6 = examples::feedback_with("(8 + 4*s)/(s*(s + 2)^2)", "1", Sign::Minus);
    let r_q6 = solve(&d_q6).unwrap();
    assert_eq!(r_q6.transfer, expr("4/(s^2 + 2*s + 4)"));

    // 4. Q7 Cascade Two-Loop Diagram:
    //    Symbolic Gc:
    let d_q7_sym = build_q7_cascade_diagram("Gc");
    let r_q7_sym = solve(&d_q7_sym).unwrap();
    assert_eq!(r_q7_sym.loops.len(), 2);
    assert_eq!(r_q7_sym.transfer, expr("(5*Gc)/(s^3 + 4*s^2 + 4*s + 5*Gc)"));
    //    Compensated with PD controller Gc(s) = 1.6 + 0.8*s:
    //    Again, exact GCD cancellation reduces (4s+8)/((s+2)(s^2+2s+4)) to 4/(s^2+2s+4)!
    let d_q7_pd = build_q7_cascade_diagram("1.6 + 0.8*s");
    let r_q7_pd = solve(&d_q7_pd).unwrap();
    assert_eq!(r_q7_pd.transfer, expr("4/(s^2 + 2*s + 4)"));

    // 5. Q8 Wind Turbine Diagram (Reference path w(s)/w_ref(s) and Disturbance path w(s)/T_p(s)):
    //    Plant G_p(s) = -7.2/(5*s + 1), Symbolic PI controller Gc(s) = (Kp*s + Ki)/s:
    let d_q8_ref = examples::feedback_with("((Kp*s + Ki)/s)*(-7.2/(5*s + 1))", "1", Sign::Minus);
    let r_q8_ref = solve(&d_q8_ref).unwrap();
    assert_eq!(
        r_q8_ref.transfer,
        expr("(-7.2*Kp*s - 7.2*Ki)/(5*s^2 + (1 - 7.2*Kp)*s - 7.2*Ki)")
    );
    //    Disturbance path (with w_ref = 0): T_p(s) -> Sum_d(+, -) -> G_p(s) -> w(s),
    //    feedback through G_c(s) into Sum_d(-):
    let d_q8_dist = examples::feedback_with("-7.2/(5*s + 1)", "(Kp*s + Ki)/s", Sign::Minus);
    let r_q8_dist = solve(&d_q8_dist).unwrap();
    assert_eq!(
        r_q8_dist.transfer,
        expr("(-7.2*s)/(5*s^2 + (1 - 7.2*Kp)*s - 7.2*Ki)")
    );

    // 6. Q9 Second-Order Block Diagram:
    //    Symbolic general second-order diagram (wn^2, 2*xi*wn, Gc):
    let d_q9_sym = build_q9_second_order_diagram("Gc", "wn^2", "2*xi*wn");
    let r_q9_sym = solve(&d_q9_sym).unwrap();
    assert_eq!(
        r_q9_sym.transfer,
        expr("(Gc*wn^2)/(s^2 + 2*xi*wn*s + Gc*wn^2)")
    );
    //    Evaluated at wn = 1, xi = 0 with PD controller Gc(s) = 4 + 2*s:
    let d_q9_pd = build_q9_second_order_diagram("4 + 2*s", "1", "0");
    let r_q9_pd = solve(&d_q9_pd).unwrap();
    assert_eq!(r_q9_pd.transfer, expr("(2*s + 4)/(s^2 + 2*s + 4)"));
}

#[test]
fn validate_q8_independent_reference_block_diagram_reduction() {
    // Etapa 10 Section 4 — Independent Q8 reference:
    // Plant G(s) = -7.2/(5s + 1), Controller Gc(s) = -1 - 0.2/s = (-s - 0.2)/s
    // Reference transfer function T(s) = Gc*G / (1 + Gc*G) simplifies exactly to 1.44/(s + 1.44).
    let d_ref = examples::feedback_with("((-s - 0.2)/s)*(-7.2/(5*s + 1))", "1", Sign::Minus);
    let r_ref = solve(&d_ref).unwrap();
    assert_eq!(r_ref.transfer, expr("1.44/(s + 1.44)"));

    // Disturbance transfer function T_d(s) = G / (1 + Gc*G) = -7.2*s / (5*s^2 + 8.2*s + 1.44)
    // = -1.44*s / ((s + 0.2)*(s + 1.44)), which has T_d(0) = 0 (complete constant disturbance rejection).
    let d_dist = examples::feedback_with("-7.2/(5*s + 1)", "(-s - 0.2)/s", Sign::Minus);
    let r_dist = solve(&d_dist).unwrap();
    assert_eq!(r_dist.transfer, expr("(-1.44*s)/((s + 0.2)*(s + 1.44))"));
}
