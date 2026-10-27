function CSX = pcb(CSX)
% matlab script created by hyp2mat
% create minimal mesh
mesh = {};
mesh.x = [0 0.0508];
mesh.y = [0 0.0381];
mesh.z = [0 0.00151638];
% add mesh
CSX = DefineRectGrid(CSX, 1, mesh);
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_DL01', 4.8, 0.02);
% board outline, layer DL01
CSX = AddBox(CSX, 'Dielectric_DL01', 100, [ 0, 0, 0], [ 0.0508, 0.0381, 0.00151638]  );
% copper
% create layer Top material
CSX = AddHyperLynxMetal2D(CSX, 'Top_copper', 0, 1.778e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.04826;0.01651];
pgon(:, end+1) = [0.04826;0.01905];
pgon(:, end+1) = [0.0254;0.01905];
pgon(:, end+1) = [0.0254;0.03175];
pgon(:, end+1) = [0.02286;0.03175];
pgon(:, end+1) = [0.02286;0.01905];
pgon(:, end+1) = [0.00254;0.01905];
pgon(:, end+1) = [0.00254;0.01651];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.00151638, pgon);
% create layer Bottom material
CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper', 0, 1.778e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.04953;0.00127];
pgon(:, end+1) = [0.04953;0.03683];
pgon(:, end+1) = [0.00127;0.03683];
pgon(:, end+1) = [0.00127;0.00127];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% devices
CSX.HyperLynxDevice = {};
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP1', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP2', 'layer_name', 'Bottom');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP3', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', 'TPTP10SQ', 'ref', 'TP4', 'layer_name', 'Bottom');
% ports
CSX.HyperLynxPort = {};
CSX.HyperLynxPort{end+1} = struct('ref', 'TP2.TP', 'xc', 0.00381, 'yc', 0.01778, 'z', 0, 'x1', 0.00331, 'y1', 0.01728, 'x2', 0.00431, 'y2', 0.01828, 'position', 'bottom', 'layer_name', 'Bottom');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP4.TP', 'xc', 0.04699, 'yc', 0.01778, 'z', 0, 'x1', 0.04649, 'y1', 0.01728, 'x2', 0.04749, 'y2', 0.01828, 'position', 'bottom', 'layer_name', 'Bottom');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP1.TP', 'xc', 0.00381, 'yc', 0.01778, 'z', 0.00151638, 'x1', 0.00331, 'y1', 0.01728, 'x2', 0.00431, 'y2', 0.01828, 'position', 'top', 'layer_name', 'Top');
CSX.HyperLynxPort{end+1} = struct('ref', 'TP3.TP', 'xc', 0.04699, 'yc', 0.01778, 'z', 0.00151638, 'x1', 0.04649, 'y1', 0.01728, 'x2', 0.04749, 'y2', 0.01828, 'position', 'top', 'layer_name', 'Top');
%not truncated
