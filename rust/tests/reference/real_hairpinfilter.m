function CSX = pcb(CSX)
% matlab script created by hyp2mat
% create minimal mesh
mesh = {};
mesh.x = [0 0.04564];
mesh.y = [0 0.04317];
mesh.z = [0 0.00151638];
% add mesh
CSX = DefineRectGrid(CSX, 1, mesh);
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_DL01', 4.8, 0.02);
% board outline, layer DL01
CSX = AddBox(CSX, 'Dielectric_DL01', 100, [ 0, 0, 0], [ 0.04564, 0.04317, 0.00151638]  );
% copper
% create layer Top material
CSX = AddHyperLynxMetal2D(CSX, 'Top_copper', 0, 1.778e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.03226;0.01016];
pgon(:, end+1) = [0.03226;0.03556];
pgon(:, end+1) = [0.03429;0.03556];
pgon(:, end+1) = [0.03429;0.01016];
pgon(:, end+1) = [0.03683;0.01016];
pgon(:, end+1) = [0.03683;0.03556];
pgon(:, end+1) = [0.03429;0.0381];
pgon(:, end+1) = [0.03226;0.0381];
pgon(:, end+1) = [0.02972;0.03556];
pgon(:, end+1) = [0.02972;0.01016];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.01397;0.01016];
pgon(:, end+1) = [0.01397;0.03556];
pgon(:, end+1) = [0.016;0.03556];
pgon(:, end+1) = [0.016;0.01016];
pgon(:, end+1) = [0.01854;0.01016];
pgon(:, end+1) = [0.01854;0.03556];
pgon(:, end+1) = [0.016;0.0381];
pgon(:, end+1) = [0.01397;0.0381];
pgon(:, end+1) = [0.01143;0.03556];
pgon(:, end+1) = [0.01143;0.01016];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.03962;0.00508];
pgon(:, end+1) = [0.03962;0.03556];
pgon(:, end+1) = [0.03708;0.03556];
pgon(:, end+1) = [0.03708;0.00508];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.02515;0.00762];
pgon(:, end+1) = [0.02769;0.01016];
pgon(:, end+1) = [0.02769;0.03556];
pgon(:, end+1) = [0.02515;0.03556];
pgon(:, end+1) = [0.02515;0.01016];
pgon(:, end+1) = [0.02311;0.01016];
pgon(:, end+1) = [0.02311;0.03556];
pgon(:, end+1) = [0.02057;0.03556];
pgon(:, end+1) = [0.02057;0.01016];
pgon(:, end+1) = [0.02311;0.00762];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.01118;0.00508];
pgon(:, end+1) = [0.01118;0.03556];
pgon(:, end+1) = [0.00864;0.03556];
pgon(:, end+1) = [0.00864;0.00508];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% create layer Bottom material
CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper', 0, 1.778e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.04463;0.00102];
pgon(:, end+1) = [0.04463;0.04215];
pgon(:, end+1) = [0.00102;0.04215];
pgon(:, end+1) = [0.00102;0.00102];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% devices
CSX.HyperLynxDevice = {};
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP1', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP2', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP3', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP4', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP5', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP6', 'layer_name', 'Bottom');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP7', 'layer_name', 'Bottom');
% ports
CSX.HyperLynxPort = {};
CSX.HyperLynxPort{end+1} = struct('ref', 'TP6.TP', 'xc', 0.00635, 'yc', 0.00381, 'z', 0, 'x1', 0.00585, 'y1', 0.00331, 'x2', 0.00685, 'y2', 0.00431, 'position', 'bottom', 'layer_name', 'Bottom');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP7.TP', 'xc', 0.04191, 'yc', 0.00381, 'z', 0, 'x1', 0.04141, 'y1', 0.00331, 'x2', 0.04241, 'y2', 0.00431, 'position', 'bottom', 'layer_name', 'Bottom');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP1.TP', 'xc', 0.00991, 'yc', 0.00635, 'z', 0.00151638, 'x1', 0.00941, 'y1', 0.00585, 'x2', 0.01041, 'y2', 0.00685, 'position', 'top', 'layer_name', 'Top');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP2.TP', 'xc', 0.0127, 'yc', 0.01143, 'z', 0.00151638, 'x1', 0.0122, 'y1', 0.01093, 'x2', 0.0132, 'y2', 0.01193, 'position', 'top', 'layer_name', 'Top');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP3.TP', 'xc', 0.02184, 'yc', 0.03429, 'z', 0.00151638, 'x1', 0.02134, 'y1', 0.03379, 'x2', 0.02234, 'y2', 0.03479, 'position', 'top', 'layer_name', 'Top');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP4.TP', 'xc', 0.03099, 'yc', 0.01143, 'z', 0.00151638, 'x1', 0.03049, 'y1', 0.01093, 'x2', 0.03149, 'y2', 0.01193, 'position', 'top', 'layer_name', 'Top');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP5.TP', 'xc', 0.03835, 'yc', 0.00635, 'z', 0.00151638, 'x1', 0.03785, 'y1', 0.00585, 'x2', 0.03885, 'y2', 0.00685, 'position', 'top', 'layer_name', 'Top');
%not truncated
