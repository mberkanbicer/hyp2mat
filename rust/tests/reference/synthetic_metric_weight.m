function CSX = pcb(CSX)
% matlab script created by hyp2mat
% create minimal mesh
mesh = {};
mesh.x = [0 0.02];
mesh.y = [0 0.02];
mesh.z = [0 0.0016];
% add mesh
CSX = DefineRectGrid(CSX, 1, mesh);
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_CORE', 4.2, 0.02);
% board outline, layer CORE
CSX = AddBox(CSX, 'Dielectric_CORE', 100, [ 0, 0, 0], [ 0.02, 0.02, 0.0016]  );
% copper
% create layer Top material
CSX = AddHyperLynxMetal2D(CSX, 'Top_copper', 0, 2.232e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.01408;0.00782];
pgon(:, end+1) = [0.01418;0.00792];
pgon(:, end+1) = [0.01418;0.00808];
pgon(:, end+1) = [0.01408;0.00818];
pgon(:, end+1) = [0.01274;0.00874];
pgon(:, end+1) = [0.01221;0.01];
pgon(:, end+1) = [0.01274;0.01126];
pgon(:, end+1) = [0.014;0.01179];
pgon(:, end+1) = [0.01526;0.01126];
pgon(:, end+1) = [0.01582;0.00992];
pgon(:, end+1) = [0.01592;0.00982];
pgon(:, end+1) = [0.01608;0.00982];
pgon(:, end+1) = [0.01618;0.00992];
pgon(:, end+1) = [0.01618;0.01008];
pgon(:, end+1) = [0.0156;0.01149];
pgon(:, end+1) = [0.01549;0.0116];
pgon(:, end+1) = [0.01408;0.01218];
pgon(:, end+1) = [0.01392;0.01218];
pgon(:, end+1) = [0.01251;0.0116];
pgon(:, end+1) = [0.0124;0.01149];
pgon(:, end+1) = [0.01182;0.01008];
pgon(:, end+1) = [0.01182;0.00992];
pgon(:, end+1) = [0.0124;0.00851];
pgon(:, end+1) = [0.01251;0.0084];
pgon(:, end+1) = [0.01392;0.00782];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.01;0.00735];
pgon(:, end+1) = [0.01046;0.00754];
pgon(:, end+1) = [0.01065;0.008];
pgon(:, end+1) = [0.01046;0.00846];
pgon(:, end+1) = [0.01;0.00865];
pgon(:, end+1) = [0.00954;0.00846];
pgon(:, end+1) = [0.00943;0.0082];
pgon(:, end+1) = [0.004;0.0082];
pgon(:, end+1) = [0.00386;0.00814];
pgon(:, end+1) = [0.0038;0.008];
pgon(:, end+1) = [0.00386;0.00786];
pgon(:, end+1) = [0.004;0.0078];
pgon(:, end+1) = [0.00943;0.0078];
pgon(:, end+1) = [0.00954;0.00754];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% create layer Bottom material
CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper', 0, 2.232e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.0106;0.0074];
pgon(:, end+1) = [0.0106;0.0086];
pgon(:, end+1) = [0.0094;0.0086];
pgon(:, end+1) = [0.0094;0.0074];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% via copper
CSX = AddMetal( CSX, 'via' );
CSX = AddCylinder(CSX, 'via', 300, [ 0.01 , 0.008 , 0 ], [ 0.01 , 0.008 , 0.0016 ], 0.0002);
% devices
CSX.HyperLynxDevice = {};
CSX.HyperLynxDevice{end+1} = struct('name', 'pad', 'ref', 'TP1', 'layer_name', 'Top');
% ports
CSX.HyperLynxPort = {};
%not truncated
