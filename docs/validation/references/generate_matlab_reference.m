% MATLAB R2022b External Reference Generator for ControLAB Academic Validation
% Covers all 19 LGR cases (Exercicios_-_LGR.pdf) and 9 Controller cases (Projeto_de_controladores.pdf)

s = tf('s');

lgr_cases = {
    struct('id', 'LGR_1a_eq01', 'question', '1(a)', 'eq', 1, 'num', [1], 'den', [1 6 25 0], 'expr', '1/(s*(s^2+6*s+25))'),
    struct('id', 'LGR_1b_eq02', 'question', '1(b)', 'eq', 2, 'num', [1 2], 'den', [1 2 3], 'expr', '(s+2)/(s^2+2*s+3)'),
    struct('id', 'LGR_1c_eq03', 'question', '1(c)', 'eq', 3, 'num', [1], 'den', [1 4 5 0], 'expr', '1/(s*(s^2+4*s+5))'),
    struct('id', 'LGR_2a_eq04', 'question', '2(a)', 'eq', 4, 'num', [1], 'den', [1 4 0], 'expr', '1/(s*(s+4))'),
    struct('id', 'LGR_2b_eq05', 'question', '2(b)', 'eq', 5, 'num', [1], 'den', [1 3 2 0], 'expr', '1/(s*(s+1)*(s+2))'),
    struct('id', 'LGR_2c_eq06', 'question', '2(c)', 'eq', 6, 'num', [1 2], 'den', [1 2 3], 'expr', '(s+2)/(s^2+2*s+3)'),
    struct('id', 'LGR_2d_eq07', 'question', '2(d)', 'eq', 7, 'num', [1], 'den', [1 6 25 0], 'expr', '1/(s*(s^2+6*s+25))'),
    struct('id', 'LGR_2e_eq08', 'question', '2(e)', 'eq', 8, 'num', [1 5 6], 'den', [1 1 0], 'expr', '(s+2)*(s+3)/(s*(s+1))'),
    struct('id', 'LGR_2f_eq09', 'question', '2(f)', 'eq', 9, 'num', [1 2 8], 'den', [1 15 50 0], 'expr', '(s^2+2*s+8)/(s*(s+10)*(s+5))'),
    struct('id', 'LGR_2g_eq10', 'question', '2(g)', 'eq', 10, 'num', [1 0 4], 'den', conv(conv([1 2],[1 2]), conv([1 5],[1 6])), 'expr', '(s^2+4)/((s+2)^2*(s+5)*(s+6))'),
    struct('id', 'LGR_2h_eq11', 'question', '2(h)', 'eq', 11, 'num', [1], 'den', conv([1 1], [1 4 5]), 'expr', '1/((s+1)*(s^2+4*s+5))'),
    struct('id', 'LGR_2i1_eq12', 'question', '2(i)-1', 'eq', 12, 'num', [1], 'den', [1 4 5 0], 'expr', '1/(s*(s^2+4*s+5))'),
    struct('id', 'LGR_2i2_eq13', 'question', '2(i)-2', 'eq', 13, 'num', [1 2], 'den', conv([1 1], [1 6 10]), 'expr', '(s+2)/((s+1)*(s^2+6*s+10))'),
    struct('id', 'LGR_3a_eq14', 'question', '3(a)', 'eq', 14, 'num', [1], 'den', [1 4 5 0], 'expr', '1/(s*(s^2+4*s+5))'),
    struct('id', 'LGR_3b_eq15', 'question', '3(b)', 'eq', 15, 'num', [1], 'den', conv([1 1 0], [1 4 13]), 'expr', '1/(s*(s+1)*(s^2+4*s+13))'),
    struct('id', 'LGR_3c_eq16', 'question', '3(c)', 'eq', 16, 'num', [1 1], 'den', [1 3.6 0 0], 'expr', '(s+1)/(s^2*(s+3.6))'),
    struct('id', 'LGR_4a_eq17', 'question', '4(a)', 'eq', 17, 'num', [1 2], 'den', [1 4 13], 'expr', '(s+2)/(s^2+4*s+13)'),
    struct('id', 'LGR_4b_eq18', 'question', '4(b)', 'eq', 18, 'num', conv([1 -2], [1 -4]), 'den', [1 6 25], 'expr', '(s-2)*(s-4)/(s^2+6*s+25)'),
    struct('id', 'LGR_5_eq19', 'question', '5(a,b)', 'eq', 19, 'num', [1 1], 'den', [1 0 0 0], 'expr', '(s+1)/s^3')
};

sample_gains = [0, 1e-3, 0.01, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 1000.0];

lgr_out = cell(1, length(lgr_cases));
for idx = 1:length(lgr_cases)
    c = lgr_cases{idx};
    sys = tf(c.num, c.den);
    p = pole(sys);
    z = zero(sys);
    P = length(p);
    Z = length(z);
    n_branches = max(P, Z);

    if P > Z
        centroid = (sum(real(p)) - sum(real(z))) / (P - Z);
        asym_angles = zeros(1, P - Z);
        for k = 0:(P - Z - 1)
            asym_angles(k + 1) = mod((2*k + 1) * 180 / (P - Z), 360);
        end
    else
        centroid = [];
        asym_angles = [];
    end

    % Breakaway polynomial: N'(s)D(s) - N(s)D'(s) = 0
    dN = polyder(c.num);
    dD = polyder(c.den);
    t1 = conv(dN, c.den);
    t2 = conv(c.num, dD);
    max_len = max(length(t1), length(t2));
    t1_pad = [zeros(1, max_len - length(t1)), t1];
    t2_pad = [zeros(1, max_len - length(t2)), t2];
    break_poly = t1_pad - t2_pad;
    b_roots = roots(break_poly);

    valid_breaks = {};
    for br_i = 1:length(b_roots)
        r = b_roots(br_i);
        if abs(imag(r)) < 1e-6
            sr = real(r);
            n_val = polyval(c.num, sr);
            d_val = polyval(c.den, sr);
            if abs(n_val) > 1e-8
                k_val = -d_val / n_val;
                if k_val >= -1e-7
                    valid_breaks{end+1} = struct('s', sr, 'K', max(0, k_val));
                end
            end
        end
    end

    % Root locus at sample gains
    r_mat = rlocus(sys, sample_gains);
    locus_samples = cell(1, length(sample_gains));
    for g_i = 1:length(sample_gains)
        col = r_mat(:, g_i);
        locus_samples{g_i} = struct('K', sample_gains(g_i), 'roots_re', real(col)', 'roots_im', imag(col)');
    end

    % Departure angles for upper-half complex poles
    dep_angles = {};
    for p_i = 1:length(p)
        pt = p(p_i);
        if imag(pt) > 1e-4
            sum_z = 0;
            for z_i = 1:length(z)
                sum_z = sum_z + atan2d(imag(pt - z(z_i)), real(pt - z(z_i)));
            end
            sum_p = 0;
            for p_j = 1:length(p)
                if abs(pt - p(p_j)) > 1e-4
                    sum_p = sum_p + atan2d(imag(pt - p(p_j)), real(pt - p(p_j)));
                end
            end
            ang = mod(180 + sum_z - sum_p + 180, 360) - 180;
            dep_angles{end+1} = struct('pole_re', real(pt), 'pole_im', imag(pt), 'angle_deg', ang);
        end
    end

    % Arrival angles for upper-half complex zeros
    arr_angles = {};
    for z_i = 1:length(z)
        zt = z(z_i);
        if imag(zt) > 1e-4
            sum_p = 0;
            for p_i = 1:length(p)
                sum_p = sum_p + atan2d(imag(zt - p(p_i)), real(zt - p(p_i)));
            end
            sum_z = 0;
            for z_j = 1:length(z)
                if abs(zt - z(z_j)) > 1e-4
                    sum_z = sum_z + atan2d(imag(zt - z(z_j)), real(zt - z(z_j)));
                end
            end
            ang = mod(180 + sum_p - sum_z + 180, 360) - 180;
            arr_angles{end+1} = struct('zero_re', real(zt), 'zero_im', imag(zt), 'angle_deg', ang);
        end
    end

    lgr_out{idx} = struct( ...
        'id', c.id, ...
        'question', c.question, ...
        'eq', c.eq, ...
        'expr', c.expr, ...
        'num', c.num, ...
        'den', c.den, ...
        'P', P, ...
        'Z', Z, ...
        'branches', n_branches, ...
        'poles_re', real(p)', ...
        'poles_im', imag(p)', ...
        'zeros_re', real(z)', ...
        'zeros_im', imag(z)', ...
        'centroid', centroid, ...
        'asymptotes_deg', asym_angles, ...
        'break_points', {valid_breaks}, ...
        'departure_angles', {dep_angles}, ...
        'arrival_angles', {arr_angles}, ...
        'locus_samples', {locus_samples} ...
    );
end

% Controller Design Questions (Projeto_de_controladores.pdf)
ctrl_cases = {};

% Q1: G(s) = 1/((s+0.2)*(s+0.644)), xi = 0.456, wn = 0.706 -> PI controller
G1 = 1 / ((s + 0.2) * (s + 0.644));
xi1 = 0.456; wn1 = 0.706;
re1 = xi1 * wn1; im1 = wn1 * sqrt(1 - xi1^2);
sd1 = -re1 + 1i * im1;
% Angle from poles (0, -0.2, -0.644)
ang_p1 = atan2d(im1, -re1) + atan2d(im1, -re1 + 0.2) + atan2d(im1, -re1 + 0.644);
theta_z1 = mod(ang_p1 + 180, 360);
z1 = re1 + im1 / tand(theta_z1);
Kp1 = abs(sd1 * (sd1 + 0.2) * (sd1 + 0.644)) / abs(sd1 + z1);
Ki1 = Kp1 * z1;
Gc1 = Kp1 * (s + z1) / s;
T1 = feedback(Gc1 * G1, 1);
info1 = stepinfo(T1, 'SettlingTimeThreshold', 0.02);
ctrl_cases{end+1} = struct('id', 'CTRL_Q1_PI', 'question', '1', 'controller_type', 'PI', ...
    'Kp', Kp1, 'Ki', Ki1, 'Kd', 0, 'z', z1, 'p', 0, ...
    'cl_poles_re', real(pole(T1))', 'cl_poles_im', imag(pole(T1))', ...
    'cl_zeros_re', real(zero(T1))', 'cl_zeros_im', imag(zero(T1))', ...
    'dcgain', dcgain(T1), 'overshoot', info1.Overshoot, ...
    'settling_time', info1.SettlingTime, 'rise_time', info1.RiseTime, 'peak_time', info1.PeakTime);

% Q2: G(s) = 1/(s^2+1), xi = 0.5, ts(2%) = 4 -> PD controller (Kp=3, Kd=2, z=1.5)
G2 = 1 / (s^2 + 1);
Kp2 = 3; Kd2 = 2; z2 = 1.5; td2 = 2/3;
Gc2 = Kp2 + Kd2 * s;
T2 = feedback(Gc2 * G2, 1);
info2 = stepinfo(T2, 'SettlingTimeThreshold', 0.02);
ctrl_cases{end+1} = struct('id', 'CTRL_Q2_PD', 'question', '2', 'controller_type', 'PD', ...
    'Kp', Kp2, 'Ki', 0, 'Kd', Kd2, 'z', z2, 'td', td2, ...
    'cl_poles_re', real(pole(T2))', 'cl_poles_im', imag(pole(T2))', ...
    'cl_zeros_re', real(zero(T2))', 'cl_zeros_im', imag(zero(T2))', ...
    'dcgain', dcgain(T2), 'overshoot', info2.Overshoot, ...
    'settling_time', info2.SettlingTime, 'rise_time', info2.RiseTime, 'peak_time', info2.PeakTime);

% Q3: Inner loop 18/((2s+11)(2s+1)) with feedback 0.5, times 1/(30s), sd = -1 +- 1j -> PD controller
G3_inner = feedback(18 / ((2*s + 11)*(2*s + 1)), 0.5);
G3 = minreal(G3_inner * (1 / (30 * s)));
Kp3 = 160/3; Kd3 = 100/3; z3 = 1.6; td3 = 0.625;
Gc3 = Kp3 + Kd3 * s;
T3 = feedback(Gc3 * G3, 1);
info3 = stepinfo(T3, 'SettlingTimeThreshold', 0.02);
ctrl_cases{end+1} = struct('id', 'CTRL_Q3_PD', 'question', '3', 'controller_type', 'PD', ...
    'Kp', Kp3, 'Ki', 0, 'Kd', Kd3, 'z', z3, 'td', td3, ...
    'cl_poles_re', real(pole(T3))', 'cl_poles_im', imag(pole(T3))', ...
    'cl_zeros_re', real(zero(T3))', 'cl_zeros_im', imag(zero(T3))', ...
    'dcgain', dcgain(T3), 'overshoot', info3.Overshoot, ...
    'settling_time', info3.SettlingTime, 'rise_time', info3.RiseTime, 'peak_time', info3.PeakTime);

% Q4: G(s) = 5/(s+0.1), H(s) = 0.05, sd = -1 +- sqrt(3)j -> PI controller
G4 = 5 / (s + 0.1);
H4 = tf(0.05);
Kp4 = 7.6; Ki4 = 16.0; z4 = 40/19;
Gc4 = Kp4 + Ki4 / s;
T4 = feedback(Gc4 * G4, H4);
info4 = stepinfo(T4, 'SettlingTimeThreshold', 0.02);
ctrl_cases{end+1} = struct('id', 'CTRL_Q4_PI', 'question', '4', 'controller_type', 'PI', ...
    'Kp', Kp4, 'Ki', Ki4, 'Kd', 0, 'z', z4, ...
    'cl_poles_re', real(pole(T4))', 'cl_poles_im', imag(pole(T4))', ...
    'cl_zeros_re', real(zero(T4))', 'cl_zeros_im', imag(zero(T4))', ...
    'dcgain', dcgain(T4), 'overshoot', info4.Overshoot, ...
    'settling_time', info4.SettlingTime, 'rise_time', info4.RiseTime, 'peak_time', info4.PeakTime);

% Q5 (Mode 1 & 2: Pole cancellation / Equal zeros z1=z2=1): G(s) = 1/(s^3+10s^2+17s+8), ts=1, xi=0.4
G5 = 1 / (s^3 + 10*s^2 + 17*s + 8);
Kp5 = 200; Ki5 = 100; Kd5 = 100;
Gc5 = (Kd5*s^2 + Kp5*s + Ki5) / s;
% Note: in ControLAB, improper Gc*G (or proper 3rd order plant + PID) -> deg(num)=2, deg(den)=4 -> strictly proper!
T5 = feedback(Gc5 * G5, 1);
T5_min = minreal(T5);
info5 = stepinfo(T5, 'SettlingTimeThreshold', 0.02);
ctrl_cases{end+1} = struct('id', 'CTRL_Q5_PID_CANCEL_EQUAL', 'question', '5', 'controller_type', 'PID', ...
    'Kp', Kp5, 'Ki', Ki5, 'Kd', Kd5, 'z1', 1.0, 'z2', 1.0, ...
    'cl_poles_re', real(pole(T5))', 'cl_poles_im', imag(pole(T5))', ...
    'cl_zeros_re', real(zero(T5))', 'cl_zeros_im', imag(zero(T5))', ...
    'dcgain', dcgain(T5), 'overshoot', info5.Overshoot, ...
    'settling_time', info5.SettlingTime, 'rise_time', info5.RiseTime, 'peak_time', info5.PeakTime);

% Q5 (Mode 3: Distinct PI-PD with zi = 0.4)
sd5 = -4 + 1i * sqrt(84);
zi5 = 0.4;
ang_p5 = atan2d(imag(sd5), real(sd5)) + 2*atan2d(imag(sd5), real(sd5)+1) + atan2d(imag(sd5), real(sd5)+8);
ang_zi5 = atan2d(imag(sd5), real(sd5)+zi5);
theta_zd5 = mod(ang_p5 - ang_zi5 + 180, 360);
zd5 = 4 + sqrt(84) / tand(theta_zd5);
K5_dist = abs(sd5 * (sd5+1)^2 * (sd5+8)) / abs((sd5+zi5)*(sd5+zd5));
Kp5_dist = K5_dist * (zi5 + zd5);
Ki5_dist = K5_dist * zi5 * zd5;
Kd5_dist = K5_dist;
Gc5_dist = (Kd5_dist*s^2 + Kp5_dist*s + Ki5_dist) / s;
T5_dist = feedback(Gc5_dist * G5, 1);
info5_dist = stepinfo(T5_dist, 'SettlingTimeThreshold', 0.02);
ctrl_cases{end+1} = struct('id', 'CTRL_Q5_PID_DISTINCT', 'question', '5', 'controller_type', 'PID', ...
    'Kp', Kp5_dist, 'Ki', Ki5_dist, 'Kd', Kd5_dist, 'z1', zi5, 'z2', zd5, ...
    'cl_poles_re', real(pole(T5_dist))', 'cl_poles_im', imag(pole(T5_dist))', ...
    'cl_zeros_re', real(zero(T5_dist))', 'cl_zeros_im', imag(zero(T5_dist))', ...
    'dcgain', dcgain(T5_dist), 'overshoot', info5_dist.Overshoot, ...
    'settling_time', info5_dist.SettlingTime, 'rise_time', info5_dist.RiseTime, 'peak_time', info5_dist.PeakTime);

% Q6: G(s) = 1/(s*(s+2)^2), PD controller, xi = 0.5, wn = 2 -> Kp = 8, Kd = 4, z = 2
G6 = 1 / (s * (s + 2)^2);
Kp6 = 8; Kd6 = 4; z6 = 2.0; td6 = 0.5;
Gc6 = Kp6 + Kd6 * s;
T6 = feedback(Gc6 * G6, 1);
T6_min = minreal(T6);
info6 = stepinfo(T6, 'SettlingTimeThreshold', 0.02);
ctrl_cases{end+1} = struct('id', 'CTRL_Q6_PD', 'question', '6', 'controller_type', 'PD', ...
    'Kp', Kp6, 'Ki', 0, 'Kd', Kd6, 'z', z6, 'td', td6, ...
    'cl_poles_re', real(pole(T6))', 'cl_poles_im', imag(pole(T6))', ...
    'cl_poles_minreal_re', real(pole(T6_min))', 'cl_poles_minreal_im', imag(pole(T6_min))', ...
    'cl_zeros_re', real(zero(T6))', 'cl_zeros_im', imag(zero(T6))', ...
    'dcgain', dcgain(T6), 'overshoot', info6.Overshoot, ...
    'settling_time', info6.SettlingTime, 'rise_time', info6.RiseTime, 'peak_time', info6.PeakTime);

% Q7: Inner loop 1/s with feedback 2 -> 1/(s+2), times 5/(s*(s+2)) -> 5/(s*(s+2)^2), xi=0.5, ts=4 -> PD
G7_inner = feedback(1/s, 2);
G7 = G7_inner * (5 / (s * (s + 2)));
Kp7 = 1.6; Kd7 = 0.8; z7 = 2.0; td7 = 0.5;
Gc7 = Kp7 + Kd7 * s;
T7 = feedback(Gc7 * G7, 1);
info7 = stepinfo(T7, 'SettlingTimeThreshold', 0.02);
ctrl_cases{end+1} = struct('id', 'CTRL_Q7_PD', 'question', '7', 'controller_type', 'PD', ...
    'Kp', Kp7, 'Ki', 0, 'Kd', Kd7, 'z', z7, 'td', td7, ...
    'cl_poles_re', real(pole(T7))', 'cl_poles_im', imag(pole(T7))', ...
    'cl_zeros_re', real(zero(T7))', 'cl_zeros_im', imag(zero(T7))', ...
    'dcgain', dcgain(T7), 'overshoot', info7.Overshoot, ...
    'settling_time', info7.SettlingTime, 'rise_time', info7.RiseTime, 'peak_time', info7.PeakTime);

% Q8: G(s) = -7.2/(5s+1), ts=4, Mp=25% -> PI controller
G8 = -7.2 / (5*s + 1);
ln_mp8 = log(0.25);
xi8 = -ln_mp8 / sqrt(pi^2 + ln_mp8^2);
wn8 = 1.0 / xi8;
Kp8 = -1.25;
z8 = wn8^2 / 1.8;
Ki8 = Kp8 * z8;
Gc8 = Kp8 + Ki8 / s;
T8 = feedback(Gc8 * G8, 1);
info8 = stepinfo(T8, 'SettlingTimeThreshold', 0.02);
ctrl_cases{end+1} = struct('id', 'CTRL_Q8_PI', 'question', '8', 'controller_type', 'PI', ...
    'Kp', Kp8, 'Ki', Ki8, 'Kd', 0, 'z', z8, 'xi', xi8, 'wn', wn8, ...
    'cl_poles_re', real(pole(T8))', 'cl_poles_im', imag(pole(T8))', ...
    'cl_zeros_re', real(zero(T8))', 'cl_zeros_im', imag(zero(T8))', ...
    'dcgain', dcgain(T8), 'overshoot', info8.Overshoot, ...
    'settling_time', info8.SettlingTime, 'rise_time', info8.RiseTime, 'peak_time', info8.PeakTime);

% Q9: G(s) = 1/s^2 (with wn=1, xi=0 in inner/forward diagram), xi=0.5, ts=4 -> PD controller (Kp=4, Kd=2, td=0.5, z=2)
G9 = 1 / s^2;
Kp9 = 4.0; Kd9 = 2.0; z9 = 2.0; td9 = 0.5;
Gc9 = Kp9 + Kd9 * s;
T9 = feedback(Gc9 * G9, 1);
info9 = stepinfo(T9, 'SettlingTimeThreshold', 0.02);
ctrl_cases{end+1} = struct('id', 'CTRL_Q9_PD', 'question', '9', 'controller_type', 'PD', ...
    'Kp', Kp9, 'Ki', 0, 'Kd', Kd9, 'z', z9, 'td', td9, ...
    'cl_poles_re', real(pole(T9))', 'cl_poles_im', imag(pole(T9))', ...
    'cl_zeros_re', real(zero(T9))', 'cl_zeros_im', imag(zero(T9))', ...
    'dcgain', dcgain(T9), 'overshoot', info9.Overshoot, ...
    'settling_time', info9.SettlingTime, 'rise_time', info9.RiseTime, 'peak_time', info9.PeakTime);

out = struct('matlab_version', version, 'lgr', {lgr_out}, 'controllers', {ctrl_cases});
json_text = jsonencode(out, 'PrettyPrint', true);
fid = fopen('matlab_r2022b_reference.json', 'w');
fwrite(fid, json_text, 'char');
fclose(fid);
disp('MATLAB reference JSON generated successfully.');
