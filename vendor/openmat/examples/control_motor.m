% openMAT: planta SISO contínua de segunda ordem e PID ideal.
% Ganhos ilustrativos; não representam sintonia automática.
s = tf('s');
G = 10 / (s^2 + 3*s + 10);
C = pid(2, 1, 0.2);
T = feedback(C*G, 1);
t = 0:0.02:8;
y = step(T, t);
ganho_estatico = dcgain(T)
% Tabela tempo/resposta; gráficos fazem parte da próxima etapa.
amostras = [t(1:10)' y(1:10)]
