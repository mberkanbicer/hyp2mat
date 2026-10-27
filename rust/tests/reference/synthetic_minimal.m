function CSX = pcb(CSX)
% matlab script created by hyp2mat
% create minimal mesh
mesh = {};
mesh.x = [0.001 0.005];
mesh.y = [0.001 0.005];
mesh.z = [0 0.001];
% add mesh
CSX = DefineRectGrid(CSX, 1, mesh);
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_DL01', 4.3, 0.02);
% board outline, layer DL01
CSX = AddBox(CSX, 'Dielectric_DL01', 100, [ 0.001, 0.001, 0], [ 0.005, 0.005, 0.001]  );
% copper
% create layer Top material
CSX = AddHyperLynxMetal2D(CSX, 'Top_copper', 0, 3.5e-06);
% copper
pgon = [];
pgon(:, end+1) = [0.005;0.001];
pgon(:, end+1) = [0.005;0.005];
pgon(:, end+1) = [0.001;0.005];
pgon(:, end+1) = [0.001;0.001];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.001, pgon);
% devices
CSX.HyperLynxDevice = {};
% ports
CSX.HyperLynxPort = {};
%not truncated
