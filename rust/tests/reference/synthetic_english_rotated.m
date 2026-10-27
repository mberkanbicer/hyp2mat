function CSX = pcb(CSX)
% matlab script created by hyp2mat
% create minimal mesh
mesh = {};
mesh.x = [0 0.00254];
mesh.y = [0 0.00203];
mesh.z = [0 0.00151638];
% add mesh
CSX = DefineRectGrid(CSX, 1, mesh);
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_DL01', 4.8, 0.02);
% board outline, layer DL01
CSX = AddBox(CSX, 'Dielectric_DL01', 100, [ 0, 0, 0], [ 0.00254, 0.00203, 0.00151638]  );
% copper
% create layer Top material
CSX = AddHyperLynxMetal2D(CSX, 'Top_copper', 0, 3.556e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.00165;0.00038];
pgon(:, end+1) = [0.00165;0.00165];
pgon(:, end+1) = [0;0.00165];
pgon(:, end+1) = [0;0.00038];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% create layer Bottom material
CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper', 0, 3.556e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.0019;0.00063];
pgon(:, end+1) = [0.0019;0.0014];
pgon(:, end+1) = [0;0.0014];
pgon(:, end+1) = [0;0.00063];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% devices
CSX.HyperLynxDevice = {};
% ports
CSX.HyperLynxPort = {};
%not truncated
