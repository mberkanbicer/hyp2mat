function CSX = pcb(CSX)
% matlab script created by hyp2mat
% create minimal mesh
mesh = {};
mesh.x = [0 0.02];
mesh.y = [0 0.02];
mesh.z = [0 0.00183617];
% add mesh
CSX = DefineRectGrid(CSX, 1, mesh);
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_DL02', 4.8, 0.02);
% board outline, layer DL02
CSX = AddBox(CSX, 'Dielectric_DL02', 100, [ 0, 0, 0], [ 0.02, 0.02, 0.00151638]  );
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_DL01', 4.8, 0.02);
% board outline, layer DL01
CSX = AddBox(CSX, 'Dielectric_DL01', 100, [ 0, 0, 0.00151638], [ 0.02, 0.02, 0.00183617]  );
% copper
% create layer Top material
CSX = AddHyperLynxMetal2D(CSX, 'Top_copper', 0, 3.556e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.00979;0.00964];
pgon(:, end+1) = [0.00979;0.01036];
pgon(:, end+1) = [0.00891;0.01036];
pgon(:, end+1) = [0.00891;0.01005];
pgon(:, end+1) = [0.00195;0.01005];
pgon(:, end+1) = [0.00195;0.00995];
pgon(:, end+1) = [0.00891;0.00995];
pgon(:, end+1) = [0.00891;0.00964];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00183617, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.01109;0.00964];
pgon(:, end+1) = [0.01109;0.00995];
pgon(:, end+1) = [0.01805;0.00995];
pgon(:, end+1) = [0.01805;0.01005];
pgon(:, end+1) = [0.01109;0.01005];
pgon(:, end+1) = [0.01109;0.01036];
pgon(:, end+1) = [0.01021;0.01036];
pgon(:, end+1) = [0.01021;0.00964];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00183617, pgon);
% create layer Bottom material
CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper', 0, 3.556e-05);
% create layer Bottom cutout
CSX = AddMaterial( CSX, 'Bottom_cutout');
CSX = SetMaterialProperty( CSX, 'Bottom_cutout', 'Epsilon', 4.8, 'Mue', 1);
% copper
pgon = [];
pgon(:, end+1) = [0.01898;0.00102];
pgon(:, end+1) = [0.01898;0.01898];
pgon(:, end+1) = [0.00102;0.01898];
pgon(:, end+1) = [0.00102;0.00102];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0.00151638, pgon);
% cutout
pgon = [];
pgon(:, end+1) = [0.0087;0.00945];
pgon(:, end+1) = [0.0087;0.01055];
pgon(:, end+1) = [0.0113;0.01055];
pgon(:, end+1) = [0.0113;0.00945];
CSX = AddPolygon(CSX, 'Bottom_cutout', 201, 2, 0.00151638, pgon);
% devices
CSX.HyperLynxDevice = {};
CSX.HyperLynxDevice{end+1} = struct('name', '', 'ref', 'C1', 'value', '100n', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP06SQ', 'ref', 'TP1', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP06SQ', 'ref', 'TP2', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP06SQ', 'ref', 'TP3', 'layer_name', 'Bottom');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP06SQ', 'ref', 'TP4', 'layer_name', 'Bottom');
% ports
CSX.HyperLynxPort = {};
CSX.HyperLynxPort{end+1} = struct('ref', 'TP3.TP', 'xc', 0.002, 'yc', 0.01, 'z', 0.00151638, 'x1', 0.00195, 'y1', 0.00995, 'x2', 0.00205, 'y2', 0.01005, 'position', 'top', 'layer_name', 'Bottom');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP4.TP', 'xc', 0.018, 'yc', 0.01, 'z', 0.00151638, 'x1', 0.01795, 'y1', 0.00995, 'x2', 0.01805, 'y2', 0.01005, 'position', 'top', 'layer_name', 'Bottom');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP1.TP', 'xc', 0.002, 'yc', 0.01, 'z', 0.00183617, 'x1', 0.00195, 'y1', 0.00995, 'x2', 0.00205, 'y2', 0.01005, 'position', 'top', 'layer_name', 'Top');
CSX.HyperLynxPort{end+1} = struct('ref', 'C1.1', 'xc', 0.00935, 'yc', 0.01, 'z', 0.00183617, 'x1', 0.00891, 'y1', 0.00964, 'x2', 0.00979, 'y2', 0.01036, 'position', 'top', 'layer_name', 'Top');
CSX.HyperLynxPort{end+1} = struct('ref', 'C1.2', 'xc', 0.01065, 'yc', 0.01, 'z', 0.00183617, 'x1', 0.01021, 'y1', 0.00964, 'x2', 0.01109, 'y2', 0.01036, 'position', 'top', 'layer_name', 'Top');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP2.TP', 'xc', 0.018, 'yc', 0.01, 'z', 0.00183617, 'x1', 0.01795, 'y1', 0.00995, 'x2', 0.01805, 'y2', 0.01005, 'position', 'top', 'layer_name', 'Top');
%not truncated
