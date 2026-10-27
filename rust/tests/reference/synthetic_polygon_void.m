function CSX = pcb(CSX)
% matlab script created by hyp2mat
% create minimal mesh
mesh = {};
mesh.x = [0 0.024];
mesh.y = [0 0.016];
mesh.z = [0 0.0016];
% add mesh
CSX = DefineRectGrid(CSX, 1, mesh);
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_DL01', 4.3, 0.02);
% board outline, layer DL01
CSX = AddBox(CSX, 'Dielectric_DL01', 100, [ 0, 0, 0], [ 0.024, 0.016, 0.0016]  );
% copper
% create layer Top material
CSX = AddHyperLynxMetal2D(CSX, 'Top_copper', 0, 7e-06);
% create layer Top cutout
CSX = AddMaterial( CSX, 'Top_cutout');
CSX = SetMaterialProperty( CSX, 'Top_cutout', 'Epsilon', 1, 'Mue', 1);
% copper
pgon = [];
pgon(:, end+1) = [0.00275;0.00125];
pgon(:, end+1) = [0.00275;0.002];
pgon(:, end+1) = [0.022;0.002];
pgon(:, end+1) = [0.022;0.014];
pgon(:, end+1) = [0.002;0.014];
pgon(:, end+1) = [0.002;0.00275];
pgon(:, end+1) = [0.00125;0.00275];
pgon(:, end+1) = [0.00125;0.00125];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% cutout
pgon = [];
pgon(:, end+1) = [0.008;0.006];
pgon(:, end+1) = [0.008;0.00785];
pgon(:, end+1) = [0.00811;0.00789];
pgon(:, end+1) = [0.00815;0.008];
pgon(:, end+1) = [0.00815;0.01];
pgon(:, end+1) = [0.014;0.01];
pgon(:, end+1) = [0.014;0.006];
CSX = AddPolygon(CSX, 'Top_cutout', 201, 2, 0.0016, pgon);
% create layer Bottom material
CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper', 0, 7e-06);
% create layer Bottom cutout
CSX = AddMaterial( CSX, 'Bottom_cutout');
CSX = SetMaterialProperty( CSX, 'Bottom_cutout', 'Epsilon', 1, 'Mue', 1);
% copper
pgon = [];
pgon(:, end+1) = [0.023;0.001];
pgon(:, end+1) = [0.023;0.015];
pgon(:, end+1) = [0.001;0.015];
pgon(:, end+1) = [0.001;0.001];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% cutout
pgon = [];
pgon(:, end+1) = [0.01;0.006];
pgon(:, end+1) = [0.01;0.01];
pgon(:, end+1) = [0.014;0.01];
pgon(:, end+1) = [0.014;0.006];
CSX = AddPolygon(CSX, 'Bottom_cutout', 201, 2, 0, pgon);
% devices
CSX.HyperLynxDevice = {};
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP06SQ', 'ref', 'TP1', 'layer_name', 'Top');
% ports
CSX.HyperLynxPort = {};
CSX.HyperLynxPort{end+1} = struct('ref', 'TP1.TP', 'xc', 0.002, 'yc', 0.002, 'z', 0.0016, 'x1', 0.00125, 'y1', 0.00125, 'x2', 0.00275, 'y2', 0.00275, 'position', 'top', 'layer_name', 'Top');
%not truncated
