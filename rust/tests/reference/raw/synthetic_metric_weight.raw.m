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
pgon(:, end+1) = [0.0158152;0.00992346];
pgon(:, end+1) = [0.0159235;0.00981522];
pgon(:, end+1) = [0.0160765;0.00981522];
pgon(:, end+1) = [0.0161848;0.00992346];
pgon(:, end+1) = [0.0161848;0.0100765];
pgon(:, end+1) = [0.015599;0.0114908];
pgon(:, end+1) = [0.0154908;0.011599];
pgon(:, end+1) = [0.0153377;0.011599];
pgon(:, end+1) = [0.0152294;0.0114908];
pgon(:, end+1) = [0.0152294;0.0113377];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0153377;0.0112294];
pgon(:, end+1) = [0.0154908;0.0112294];
pgon(:, end+1) = [0.015599;0.0113377];
pgon(:, end+1) = [0.015599;0.0114908];
pgon(:, end+1) = [0.0154908;0.011599];
pgon(:, end+1) = [0.0140765;0.0121848];
pgon(:, end+1) = [0.0139235;0.0121848];
pgon(:, end+1) = [0.0138152;0.0120765];
pgon(:, end+1) = [0.0138152;0.0119235];
pgon(:, end+1) = [0.0139235;0.0118152];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0140765;0.0118152];
pgon(:, end+1) = [0.0141848;0.0119235];
pgon(:, end+1) = [0.0141848;0.0120765];
pgon(:, end+1) = [0.0140765;0.0121848];
pgon(:, end+1) = [0.0139235;0.0121848];
pgon(:, end+1) = [0.0125092;0.011599];
pgon(:, end+1) = [0.012401;0.0114908];
pgon(:, end+1) = [0.012401;0.0113377];
pgon(:, end+1) = [0.0125092;0.0112294];
pgon(:, end+1) = [0.0126623;0.0112294];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0127706;0.0113377];
pgon(:, end+1) = [0.0127706;0.0114908];
pgon(:, end+1) = [0.0126623;0.011599];
pgon(:, end+1) = [0.0125092;0.011599];
pgon(:, end+1) = [0.012401;0.0114908];
pgon(:, end+1) = [0.0118152;0.0100765];
pgon(:, end+1) = [0.0118152;0.00992346];
pgon(:, end+1) = [0.0119235;0.00981522];
pgon(:, end+1) = [0.0120765;0.00981522];
pgon(:, end+1) = [0.0121848;0.00992346];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0121848;0.0100765];
pgon(:, end+1) = [0.0120765;0.0101848];
pgon(:, end+1) = [0.0119235;0.0101848];
pgon(:, end+1) = [0.0118152;0.0100765];
pgon(:, end+1) = [0.0118152;0.00992346];
pgon(:, end+1) = [0.012401;0.00850925];
pgon(:, end+1) = [0.0125092;0.00840101];
pgon(:, end+1) = [0.0126623;0.00840101];
pgon(:, end+1) = [0.0127706;0.00850925];
pgon(:, end+1) = [0.0127706;0.00866232];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0126623;0.00877056];
pgon(:, end+1) = [0.0125092;0.00877056];
pgon(:, end+1) = [0.012401;0.00866232];
pgon(:, end+1) = [0.012401;0.00850925];
pgon(:, end+1) = [0.0125092;0.00840101];
pgon(:, end+1) = [0.0139235;0.00781522];
pgon(:, end+1) = [0.0140765;0.00781522];
pgon(:, end+1) = [0.0141848;0.00792346];
pgon(:, end+1) = [0.0141848;0.00807654];
pgon(:, end+1) = [0.0140765;0.00818478];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.01;0.0078];
pgon(:, end+1) = [0.0101414;0.00785858];
pgon(:, end+1) = [0.0102;0.008];
pgon(:, end+1) = [0.0101414;0.00814142];
pgon(:, end+1) = [0.01;0.0082];
pgon(:, end+1) = [0.004;0.0082];
pgon(:, end+1) = [0.00385858;0.00814142];
pgon(:, end+1) = [0.0038;0.008];
pgon(:, end+1) = [0.00385858;0.00785858];
pgon(:, end+1) = [0.004;0.0078];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.012;0.008];
pgon(:, end+1) = [0.012;0.008];
pgon(:, end+1) = [0.012;0.008];
pgon(:, end+1) = [0.012;0.008];
pgon(:, end+1) = [0.012;0.008];
pgon(:, end+1) = [0.012;0.008];
pgon(:, end+1) = [0.012;0.008];
pgon(:, end+1) = [0.012;0.008];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0106494;0.008];
pgon(:, end+1) = [0.0104592;0.00845922];
pgon(:, end+1) = [0.01;0.00864944];
pgon(:, end+1) = [0.00954078;0.00845922];
pgon(:, end+1) = [0.00935056;0.008];
pgon(:, end+1) = [0.00954078;0.00754078];
pgon(:, end+1) = [0.01;0.00735056];
pgon(:, end+1) = [0.0104592;0.00754078];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% create layer Bottom material
CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper', 0, 2.232e-05);
% copper
pgon = [];
pgon(:, end+1) = [0.0094;0.0086];
pgon(:, end+1) = [0.0094;0.0074];
pgon(:, end+1) = [0.0106;0.0074];
pgon(:, end+1) = [0.0106;0.0086];
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
