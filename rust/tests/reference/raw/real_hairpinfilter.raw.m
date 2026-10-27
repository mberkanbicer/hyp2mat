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
pgon(:, end+1) = [0.011176;0.03556];
pgon(:, end+1) = [0.008636;0.03556];
pgon(:, end+1) = [0.008636;0.00508];
pgon(:, end+1) = [0.011176;0.00508];
pgon(:, end+1) = [0.011176;0.03556];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0094107;0.0068453];
pgon(:, end+1) = [0.0094107;0.0058547];
pgon(:, end+1) = [0.0104013;0.0058547];
pgon(:, end+1) = [0.0104013;0.0068453];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.018542;0.03556];
pgon(:, end+1) = [0.016002;0.0381];
pgon(:, end+1) = [0.01397;0.0381];
pgon(:, end+1) = [0.01143;0.03556];
pgon(:, end+1) = [0.01143;0.01016];
pgon(:, end+1) = [0.01397;0.01016];
pgon(:, end+1) = [0.01397;0.03556];
pgon(:, end+1) = [0.016002;0.03556];
pgon(:, end+1) = [0.016002;0.01016];
pgon(:, end+1) = [0.018542;0.01016];
pgon(:, end+1) = [0.018542;0.03556];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0122047;0.0119253];
pgon(:, end+1) = [0.0122047;0.0109347];
pgon(:, end+1) = [0.0131953;0.0109347];
pgon(:, end+1) = [0.0131953;0.0119253];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.020574;0.01016];
pgon(:, end+1) = [0.023114;0.00762];
pgon(:, end+1) = [0.025146;0.00762];
pgon(:, end+1) = [0.027686;0.01016];
pgon(:, end+1) = [0.027686;0.03556];
pgon(:, end+1) = [0.025146;0.03556];
pgon(:, end+1) = [0.025146;0.01016];
pgon(:, end+1) = [0.023114;0.01016];
pgon(:, end+1) = [0.023114;0.03556];
pgon(:, end+1) = [0.020574;0.03556];
pgon(:, end+1) = [0.020574;0.01016];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0213487;0.0347853];
pgon(:, end+1) = [0.0213487;0.0337947];
pgon(:, end+1) = [0.0223393;0.0337947];
pgon(:, end+1) = [0.0223393;0.0347853];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.03683;0.03556];
pgon(:, end+1) = [0.03429;0.0381];
pgon(:, end+1) = [0.032258;0.0381];
pgon(:, end+1) = [0.029718;0.03556];
pgon(:, end+1) = [0.029718;0.01016];
pgon(:, end+1) = [0.032258;0.01016];
pgon(:, end+1) = [0.032258;0.03556];
pgon(:, end+1) = [0.03429;0.03556];
pgon(:, end+1) = [0.03429;0.01016];
pgon(:, end+1) = [0.03683;0.01016];
pgon(:, end+1) = [0.03683;0.03556];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0304927;0.0119253];
pgon(:, end+1) = [0.0304927;0.0109347];
pgon(:, end+1) = [0.0314833;0.0109347];
pgon(:, end+1) = [0.0314833;0.0119253];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.039624;0.03556];
pgon(:, end+1) = [0.037084;0.03556];
pgon(:, end+1) = [0.037084;0.00508];
pgon(:, end+1) = [0.039624;0.00508];
pgon(:, end+1) = [0.039624;0.03556];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0378587;0.0068453];
pgon(:, end+1) = [0.0378587;0.0058547];
pgon(:, end+1) = [0.0388493;0.0058547];
pgon(:, end+1) = [0.0388493;0.0068453];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% create layer Bottom material
CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper', 0, 1.778e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.0446278;0.0421538];
pgon(:, end+1) = [0.001016;0.0421538];
pgon(:, end+1) = [0.001016;0.001016];
pgon(:, end+1) = [0.0446278;0.001016];
pgon(:, end+1) = [0.0446278;0.0421538];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0414147;0.0043053];
pgon(:, end+1) = [0.0414147;0.0033147];
pgon(:, end+1) = [0.0424053;0.0033147];
pgon(:, end+1) = [0.0424053;0.0043053];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0058547;0.0043053];
pgon(:, end+1) = [0.0058547;0.0033147];
pgon(:, end+1) = [0.0068453;0.0033147];
pgon(:, end+1) = [0.0068453;0.0043053];
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
