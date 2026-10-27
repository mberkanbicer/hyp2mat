function CSX = pcb(CSX)
% matlab script created by hyp2mat
% create minimal mesh
mesh = {};
mesh.x = [0 0.03];
mesh.y = [0 0.02];
mesh.z = [0 0.0016];
% add mesh
CSX = DefineRectGrid(CSX, 1, mesh);
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_PP2', 4.4, 0.02);
% board outline, layer PP2
CSX = AddBox(CSX, 'Dielectric_PP2', 100, [ 0, 0, 0], [ 0.03, 0.02, 0.0004]  );
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_CORE', 4.6, 0.015);
% board outline, layer CORE
CSX = AddBox(CSX, 'Dielectric_CORE', 100, [ 0, 0, 0.0004], [ 0.03, 0.02, 0.0012]  );
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_PP1', 4.4, 0.02);
% board outline, layer PP1
CSX = AddBox(CSX, 'Dielectric_PP1', 100, [ 0, 0, 0.0012], [ 0.03, 0.02, 0.0016]  );
% copper
% create layer Top material
CSX = AddHyperLynxMetal2D(CSX, 'Top_copper', 0, 3.5e-06);
% copper
pgon = [];
pgon(:, end+1) = [0.0139076;0.00396173];
pgon(:, end+1) = [0.0139617;0.00390761];
pgon(:, end+1) = [0.0140383;0.00390761];
pgon(:, end+1) = [0.0140924;0.00396173];
pgon(:, end+1) = [0.0140924;0.00403827];
pgon(:, end+1) = [0.0135066;0.00545248];
pgon(:, end+1) = [0.0134525;0.0055066];
pgon(:, end+1) = [0.0133759;0.0055066];
pgon(:, end+1) = [0.0133218;0.00545248];
pgon(:, end+1) = [0.0133218;0.00537595];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0133759;0.00532183];
pgon(:, end+1) = [0.0134525;0.00532183];
pgon(:, end+1) = [0.0135066;0.00537595];
pgon(:, end+1) = [0.0135066;0.00545248];
pgon(:, end+1) = [0.0134525;0.0055066];
pgon(:, end+1) = [0.0120383;0.00609239];
pgon(:, end+1) = [0.0119617;0.00609239];
pgon(:, end+1) = [0.0119076;0.00603827];
pgon(:, end+1) = [0.0119076;0.00596173];
pgon(:, end+1) = [0.0119617;0.00590761];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0120383;0.00590761];
pgon(:, end+1) = [0.0120924;0.00596173];
pgon(:, end+1) = [0.0120924;0.00603827];
pgon(:, end+1) = [0.0120383;0.00609239];
pgon(:, end+1) = [0.0119617;0.00609239];
pgon(:, end+1) = [0.0105475;0.0055066];
pgon(:, end+1) = [0.0104934;0.00545248];
pgon(:, end+1) = [0.0104934;0.00537595];
pgon(:, end+1) = [0.0105475;0.00532183];
pgon(:, end+1) = [0.0106241;0.00532183];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0106782;0.00537595];
pgon(:, end+1) = [0.0106782;0.00545248];
pgon(:, end+1) = [0.0106241;0.0055066];
pgon(:, end+1) = [0.0105475;0.0055066];
pgon(:, end+1) = [0.0104934;0.00545248];
pgon(:, end+1) = [0.00990761;0.00403827];
pgon(:, end+1) = [0.00990761;0.00396173];
pgon(:, end+1) = [0.00996173;0.00390761];
pgon(:, end+1) = [0.0100383;0.00390761];
pgon(:, end+1) = [0.0100924;0.00396173];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0100924;0.00403827];
pgon(:, end+1) = [0.0100383;0.00409239];
pgon(:, end+1) = [0.00996173;0.00409239];
pgon(:, end+1) = [0.00990761;0.00403827];
pgon(:, end+1) = [0.00990761;0.00396173];
pgon(:, end+1) = [0.0104934;0.00254752];
pgon(:, end+1) = [0.0105475;0.0024934];
pgon(:, end+1) = [0.0106241;0.0024934];
pgon(:, end+1) = [0.0106782;0.00254752];
pgon(:, end+1) = [0.0106782;0.00262405];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0106241;0.00267817];
pgon(:, end+1) = [0.0105475;0.00267817];
pgon(:, end+1) = [0.0104934;0.00262405];
pgon(:, end+1) = [0.0104934;0.00254752];
pgon(:, end+1) = [0.0105475;0.0024934];
pgon(:, end+1) = [0.0119617;0.00190761];
pgon(:, end+1) = [0.0120383;0.00190761];
pgon(:, end+1) = [0.0120924;0.00196173];
pgon(:, end+1) = [0.0120924;0.00203827];
pgon(:, end+1) = [0.0120383;0.00209239];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.012;0.0019];
pgon(:, end+1) = [0.0120707;0.00192929];
pgon(:, end+1) = [0.0121;0.002];
pgon(:, end+1) = [0.0120707;0.00207071];
pgon(:, end+1) = [0.012;0.0021];
pgon(:, end+1) = [0.006;0.0021];
pgon(:, end+1) = [0.00592929;0.00207071];
pgon(:, end+1) = [0.0059;0.002];
pgon(:, end+1) = [0.00592929;0.00192929];
pgon(:, end+1) = [0.006;0.0019];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0156;0.0024];
pgon(:, end+1) = [0.0156;0.0016];
pgon(:, end+1) = [0.0164;0.0016];
pgon(:, end+1) = [0.0164;0.0024];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0016;0.0024];
pgon(:, end+1) = [0.0016;0.0016];
pgon(:, end+1) = [0.0024;0.0016];
pgon(:, end+1) = [0.0024;0.0024];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0236;0.0024];
pgon(:, end+1) = [0.0236;0.0016];
pgon(:, end+1) = [0.0244;0.0016];
pgon(:, end+1) = [0.0244;0.0024];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.02;0.01485];
pgon(:, end+1) = [0.0201061;0.0148939];
pgon(:, end+1) = [0.02015;0.015];
pgon(:, end+1) = [0.0201061;0.0151061];
pgon(:, end+1) = [0.02;0.01515];
pgon(:, end+1) = [0.008;0.01515];
pgon(:, end+1) = [0.00789393;0.0151061];
pgon(:, end+1) = [0.00785;0.015];
pgon(:, end+1) = [0.00789393;0.0148939];
pgon(:, end+1) = [0.008;0.01485];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0206;0.0154];
pgon(:, end+1) = [0.0206;0.0146];
pgon(:, end+1) = [0.0214;0.0146];
pgon(:, end+1) = [0.0214;0.0154];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% create layer GND material
CSX = AddHyperLynxMetal2D(CSX, 'GND_copper', 0, 3.5e-06);
% copper
pgon = [];
pgon(:, end+1) = [0.002;0.012];
pgon(:, end+1) = [0.028;0.012];
pgon(:, end+1) = [0.028;0.018];
pgon(:, end+1) = [0.002;0.018];
pgon(:, end+1) = [0.002;0.012];
CSX = AddPolygon(CSX, 'GND_copper', 200, 2, 0.0012, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0163444;0.00234442];
pgon(:, end+1) = [0.016;0.00248708];
pgon(:, end+1) = [0.0156556;0.00234442];
pgon(:, end+1) = [0.0155129;0.002];
pgon(:, end+1) = [0.0156556;0.00165558];
pgon(:, end+1) = [0.016;0.00151292];
pgon(:, end+1) = [0.0163444;0.00165558];
pgon(:, end+1) = [0.0164871;0.002];
CSX = AddPolygon(CSX, 'GND_copper', 200, 2, 0.0012, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.00234442;0.00234442];
pgon(:, end+1) = [0.002;0.00248708];
pgon(:, end+1) = [0.00165558;0.00234442];
pgon(:, end+1) = [0.00151292;0.002];
pgon(:, end+1) = [0.00165558;0.00165558];
pgon(:, end+1) = [0.002;0.00151292];
pgon(:, end+1) = [0.00234442;0.00165558];
pgon(:, end+1) = [0.00248708;0.002];
CSX = AddPolygon(CSX, 'GND_copper', 200, 2, 0.0012, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0243444;0.00234442];
pgon(:, end+1) = [0.024;0.00248708];
pgon(:, end+1) = [0.0236556;0.00234442];
pgon(:, end+1) = [0.0235129;0.002];
pgon(:, end+1) = [0.0236556;0.00165558];
pgon(:, end+1) = [0.024;0.00151292];
pgon(:, end+1) = [0.0243444;0.00165558];
pgon(:, end+1) = [0.0244871;0.002];
CSX = AddPolygon(CSX, 'GND_copper', 200, 2, 0.0012, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0213444;0.0153444];
pgon(:, end+1) = [0.021;0.0154871];
pgon(:, end+1) = [0.0206556;0.0153444];
pgon(:, end+1) = [0.0205129;0.015];
pgon(:, end+1) = [0.0206556;0.0146556];
pgon(:, end+1) = [0.021;0.0145129];
pgon(:, end+1) = [0.0213444;0.0146556];
pgon(:, end+1) = [0.0214871;0.015];
CSX = AddPolygon(CSX, 'GND_copper', 200, 2, 0.0012, pgon);
% create layer PWR material
CSX = AddHyperLynxMetal2D(CSX, 'PWR_copper', 0, 3.5e-06);
% copper
pgon = [];
pgon(:, end+1) = [0.0163444;0.00234442];
pgon(:, end+1) = [0.016;0.00248708];
pgon(:, end+1) = [0.0156556;0.00234442];
pgon(:, end+1) = [0.0155129;0.002];
pgon(:, end+1) = [0.0156556;0.00165558];
pgon(:, end+1) = [0.016;0.00151292];
pgon(:, end+1) = [0.0163444;0.00165558];
pgon(:, end+1) = [0.0164871;0.002];
CSX = AddPolygon(CSX, 'PWR_copper', 200, 2, 0.0004, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.00234442;0.00234442];
pgon(:, end+1) = [0.002;0.00248708];
pgon(:, end+1) = [0.00165558;0.00234442];
pgon(:, end+1) = [0.00151292;0.002];
pgon(:, end+1) = [0.00165558;0.00165558];
pgon(:, end+1) = [0.002;0.00151292];
pgon(:, end+1) = [0.00234442;0.00165558];
pgon(:, end+1) = [0.00248708;0.002];
CSX = AddPolygon(CSX, 'PWR_copper', 200, 2, 0.0004, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.002;0.002];
pgon(:, end+1) = [0.028;0.002];
pgon(:, end+1) = [0.028;0.008];
pgon(:, end+1) = [0.002;0.008];
pgon(:, end+1) = [0.002;0.002];
CSX = AddPolygon(CSX, 'PWR_copper', 200, 2, 0.0004, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0243444;0.00234442];
pgon(:, end+1) = [0.024;0.00248708];
pgon(:, end+1) = [0.0236556;0.00234442];
pgon(:, end+1) = [0.0235129;0.002];
pgon(:, end+1) = [0.0236556;0.00165558];
pgon(:, end+1) = [0.024;0.00151292];
pgon(:, end+1) = [0.0243444;0.00165558];
pgon(:, end+1) = [0.0244871;0.002];
CSX = AddPolygon(CSX, 'PWR_copper', 200, 2, 0.0004, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0213444;0.0153444];
pgon(:, end+1) = [0.021;0.0154871];
pgon(:, end+1) = [0.0206556;0.0153444];
pgon(:, end+1) = [0.0205129;0.015];
pgon(:, end+1) = [0.0206556;0.0146556];
pgon(:, end+1) = [0.021;0.0145129];
pgon(:, end+1) = [0.0213444;0.0146556];
pgon(:, end+1) = [0.0214871;0.015];
CSX = AddPolygon(CSX, 'PWR_copper', 200, 2, 0.0004, pgon);
% create layer Bottom material
CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper', 0, 3.5e-06);
% copper
pgon = [];
pgon(:, end+1) = [0.0164;0.002];
pgon(:, end+1) = [0.0162828;0.00228284];
pgon(:, end+1) = [0.016;0.0024];
pgon(:, end+1) = [0.0157172;0.00228284];
pgon(:, end+1) = [0.0156;0.002];
pgon(:, end+1) = [0.0156;0.002];
pgon(:, end+1) = [0.0157172;0.00171716];
pgon(:, end+1) = [0.016;0.0016];
pgon(:, end+1) = [0.0162828;0.00171716];
pgon(:, end+1) = [0.0164;0.002];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0024;0.002];
pgon(:, end+1) = [0.00228284;0.00228284];
pgon(:, end+1) = [0.002;0.0024];
pgon(:, end+1) = [0.00171716;0.00228284];
pgon(:, end+1) = [0.0016;0.002];
pgon(:, end+1) = [0.0016;0.002];
pgon(:, end+1) = [0.00171716;0.00171716];
pgon(:, end+1) = [0.002;0.0016];
pgon(:, end+1) = [0.00228284;0.00171716];
pgon(:, end+1) = [0.0024;0.002];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0244;0.002];
pgon(:, end+1) = [0.0242828;0.00228284];
pgon(:, end+1) = [0.024;0.0024];
pgon(:, end+1) = [0.0237172;0.00228284];
pgon(:, end+1) = [0.0236;0.002];
pgon(:, end+1) = [0.0236;0.002];
pgon(:, end+1) = [0.0237172;0.00171716];
pgon(:, end+1) = [0.024;0.0016];
pgon(:, end+1) = [0.0242828;0.00171716];
pgon(:, end+1) = [0.0244;0.002];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0214;0.015];
pgon(:, end+1) = [0.0212828;0.0152828];
pgon(:, end+1) = [0.021;0.0154];
pgon(:, end+1) = [0.0207172;0.0152828];
pgon(:, end+1) = [0.0206;0.015];
pgon(:, end+1) = [0.0206;0.015];
pgon(:, end+1) = [0.0207172;0.0147172];
pgon(:, end+1) = [0.021;0.0146];
pgon(:, end+1) = [0.0212828;0.0147172];
pgon(:, end+1) = [0.0214;0.015];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% via copper
CSX = AddMetal( CSX, 'via' );
CSX = AddCylinder(CSX, 'via', 300, [ 0.002 , 0.002 , 0 ], [ 0.002 , 0.002 , 0.0016 ], 0.0002);
CSX = AddCylinder(CSX, 'via', 300, [ 0.016 , 0.002 , 0 ], [ 0.016 , 0.002 , 0.0016 ], 0.0002);
CSX = AddCylinder(CSX, 'via', 300, [ 0.024 , 0.002 , 0 ], [ 0.024 , 0.002 , 0.0016 ], 0.0002);
CSX = AddCylinder(CSX, 'via', 300, [ 0.021 , 0.015 , 0 ], [ 0.021 , 0.015 , 0.0016 ], 0.0002);
% devices
CSX.HyperLynxDevice = {};
CSX.HyperLynxDevice{end+1} = struct('name', '0603', 'ref', 'R1', 'value', '50', 'layer_name', 'Top');
CSX.HyperLynxDevice{end+1} = struct('name', '0603', 'ref', 'C1', 'value', '1n', 'layer_name', 'Top');
% ports
CSX.HyperLynxPort = {};
CSX.HyperLynxPort{end+1} = struct('ref', 'R1.1', 'xc', 0.002, 'yc', 0.002, 'z', 0.0016, 'x1', 0.0016, 'y1', 0.0016, 'x2', 0.0024, 'y2', 0.0024, 'position', 'top', 'layer_name', 'Top');
CSX.HyperLynxPort{end+1} = struct('ref', 'C1.1', 'xc', 0.021, 'yc', 0.015, 'z', 0.0016, 'x1', 0.0206, 'y1', 0.0146, 'x2', 0.0214, 'y2', 0.0154, 'position', 'top', 'layer_name', 'Top');
%not truncated
