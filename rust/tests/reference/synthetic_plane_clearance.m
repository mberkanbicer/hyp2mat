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
pgon(:, end+1) = [0.0214;0.0146];
pgon(:, end+1) = [0.0214;0.0154];
pgon(:, end+1) = [0.0206;0.0154];
pgon(:, end+1) = [0.0206;0.0146];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.02;0.01485];
pgon(:, end+1) = [0.02011;0.01489];
pgon(:, end+1) = [0.02015;0.015];
pgon(:, end+1) = [0.02011;0.01511];
pgon(:, end+1) = [0.02;0.01515];
pgon(:, end+1) = [0.008;0.01515];
pgon(:, end+1) = [0.00789;0.01511];
pgon(:, end+1) = [0.00785;0.015];
pgon(:, end+1) = [0.00789;0.01489];
pgon(:, end+1) = [0.008;0.01485];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.012;0.0019];
pgon(:, end+1) = [0.01202;0.00191];
pgon(:, end+1) = [0.01204;0.00191];
pgon(:, end+1) = [0.01205;0.00192];
pgon(:, end+1) = [0.01207;0.00193];
pgon(:, end+1) = [0.01208;0.00195];
pgon(:, end+1) = [0.01209;0.00196];
pgon(:, end+1) = [0.01209;0.00198];
pgon(:, end+1) = [0.0121;0.002];
pgon(:, end+1) = [0.01209;0.00202];
pgon(:, end+1) = [0.01209;0.00204];
pgon(:, end+1) = [0.01208;0.00205];
pgon(:, end+1) = [0.01207;0.00207];
pgon(:, end+1) = [0.01205;0.00208];
pgon(:, end+1) = [0.01204;0.00209];
pgon(:, end+1) = [0.01066;0.00266];
pgon(:, end+1) = [0.01011;0.004];
pgon(:, end+1) = [0.01066;0.00534];
pgon(:, end+1) = [0.012;0.00589];
pgon(:, end+1) = [0.01334;0.00534];
pgon(:, end+1) = [0.01391;0.00396];
pgon(:, end+1) = [0.01396;0.00391];
pgon(:, end+1) = [0.01404;0.00391];
pgon(:, end+1) = [0.01409;0.00396];
pgon(:, end+1) = [0.01409;0.00404];
pgon(:, end+1) = [0.01351;0.00545];
pgon(:, end+1) = [0.01345;0.00551];
pgon(:, end+1) = [0.01204;0.00609];
pgon(:, end+1) = [0.01196;0.00609];
pgon(:, end+1) = [0.01055;0.00551];
pgon(:, end+1) = [0.01049;0.00545];
pgon(:, end+1) = [0.00991;0.00404];
pgon(:, end+1) = [0.00991;0.00396];
pgon(:, end+1) = [0.01049;0.00255];
pgon(:, end+1) = [0.01055;0.00249];
pgon(:, end+1) = [0.0115;0.0021];
pgon(:, end+1) = [0.006;0.0021];
pgon(:, end+1) = [0.00593;0.00207];
pgon(:, end+1) = [0.0059;0.002];
pgon(:, end+1) = [0.00593;0.00193];
pgon(:, end+1) = [0.006;0.0019];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0024;0.0016];
pgon(:, end+1) = [0.0024;0.0024];
pgon(:, end+1) = [0.0016;0.0024];
pgon(:, end+1) = [0.0016;0.0016];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0164;0.0016];
pgon(:, end+1) = [0.0164;0.0024];
pgon(:, end+1) = [0.0156;0.0024];
pgon(:, end+1) = [0.0156;0.0016];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.0244;0.0016];
pgon(:, end+1) = [0.0244;0.0024];
pgon(:, end+1) = [0.0236;0.0024];
pgon(:, end+1) = [0.0236;0.0016];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0016, pgon);
% create layer GND material
CSX = AddHyperLynxMetal2D(CSX, 'GND_copper', 0, 3.5e-06);
% create layer GND cutout
CSX = AddMaterial( CSX, 'GND_cutout');
CSX = SetMaterialProperty( CSX, 'GND_cutout', 'Epsilon', 4.3, 'Mue', 1);
% copper
pgon = [];
pgon(:, end+1) = [0.02802;0.01195];
pgon(:, end+1) = [0.02805;0.01197];
pgon(:, end+1) = [0.02805;0.01802];
pgon(:, end+1) = [0.02802;0.01805];
pgon(:, end+1) = [0.00197;0.01805];
pgon(:, end+1) = [0.00195;0.01802];
pgon(:, end+1) = [0.00195;0.01197];
pgon(:, end+1) = [0.00197;0.01195];
CSX = AddPolygon(CSX, 'GND_copper', 200, 2, 0.0012, pgon);
% cutout
pgon = [];
pgon(:, end+1) = [0.02095;0.01431];
pgon(:, end+1) = [0.02054;0.01449];
pgon(:, end+1) = [0.02049;0.01454];
pgon(:, end+1) = [0.02031;0.01495];
pgon(:, end+1) = [0.02031;0.01504];
pgon(:, end+1) = [0.02049;0.01545];
pgon(:, end+1) = [0.02054;0.0155];
pgon(:, end+1) = [0.02095;0.01568];
pgon(:, end+1) = [0.02104;0.01568];
pgon(:, end+1) = [0.02145;0.0155];
pgon(:, end+1) = [0.0215;0.01545];
pgon(:, end+1) = [0.02168;0.01504];
pgon(:, end+1) = [0.02168;0.01495];
pgon(:, end+1) = [0.0215;0.01454];
pgon(:, end+1) = [0.02145;0.01449];
pgon(:, end+1) = [0.02104;0.01431];
CSX = AddPolygon(CSX, 'GND_cutout', 201, 2, 0.0012, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.021;0.01451];
pgon(:, end+1) = [0.02134;0.01466];
pgon(:, end+1) = [0.02149;0.015];
pgon(:, end+1) = [0.02134;0.01534];
pgon(:, end+1) = [0.021;0.01549];
pgon(:, end+1) = [0.02066;0.01534];
pgon(:, end+1) = [0.02051;0.015];
pgon(:, end+1) = [0.02066;0.01466];
CSX = AddPolygon(CSX, 'GND_copper', 202, 2, 0.0012, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.002;0.00151];
pgon(:, end+1) = [0.00234;0.00166];
pgon(:, end+1) = [0.00249;0.002];
pgon(:, end+1) = [0.00234;0.00234];
pgon(:, end+1) = [0.002;0.00249];
pgon(:, end+1) = [0.00166;0.00234];
pgon(:, end+1) = [0.00151;0.002];
pgon(:, end+1) = [0.00166;0.00166];
CSX = AddPolygon(CSX, 'GND_copper', 200, 2, 0.0012, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.016;0.00151];
pgon(:, end+1) = [0.01634;0.00166];
pgon(:, end+1) = [0.01649;0.002];
pgon(:, end+1) = [0.01634;0.00234];
pgon(:, end+1) = [0.016;0.00249];
pgon(:, end+1) = [0.01566;0.00234];
pgon(:, end+1) = [0.01551;0.002];
pgon(:, end+1) = [0.01566;0.00166];
CSX = AddPolygon(CSX, 'GND_copper', 200, 2, 0.0012, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.024;0.00151];
pgon(:, end+1) = [0.02434;0.00166];
pgon(:, end+1) = [0.02449;0.002];
pgon(:, end+1) = [0.02434;0.00234];
pgon(:, end+1) = [0.024;0.00249];
pgon(:, end+1) = [0.02366;0.00234];
pgon(:, end+1) = [0.02351;0.002];
pgon(:, end+1) = [0.02366;0.00166];
CSX = AddPolygon(CSX, 'GND_copper', 200, 2, 0.0012, pgon);
% create layer PWR material
CSX = AddHyperLynxMetal2D(CSX, 'PWR_copper', 0, 3.5e-06);
% create layer PWR cutout
CSX = AddMaterial( CSX, 'PWR_cutout');
CSX = SetMaterialProperty( CSX, 'PWR_cutout', 'Epsilon', 4.3, 'Mue', 1);
% copper
pgon = [];
pgon(:, end+1) = [0.021;0.01451];
pgon(:, end+1) = [0.02134;0.01466];
pgon(:, end+1) = [0.02149;0.015];
pgon(:, end+1) = [0.02134;0.01534];
pgon(:, end+1) = [0.021;0.01549];
pgon(:, end+1) = [0.02066;0.01534];
pgon(:, end+1) = [0.02051;0.015];
pgon(:, end+1) = [0.02066;0.01466];
CSX = AddPolygon(CSX, 'PWR_copper', 200, 2, 0.0004, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.024;0.00151];
pgon(:, end+1) = [0.02434;0.00166];
pgon(:, end+1) = [0.02449;0.002];
pgon(:, end+1) = [0.028;0.002];
pgon(:, end+1) = [0.028;0.008];
pgon(:, end+1) = [0.002;0.008];
pgon(:, end+1) = [0.002;0.00268];
pgon(:, end+1) = [0.00204;0.00268];
pgon(:, end+1) = [0.00245;0.0025];
pgon(:, end+1) = [0.0025;0.00245];
pgon(:, end+1) = [0.00268;0.00204];
pgon(:, end+1) = [0.00268;0.002];
pgon(:, end+1) = [0.01531;0.002];
pgon(:, end+1) = [0.01531;0.00204];
pgon(:, end+1) = [0.01549;0.00245];
pgon(:, end+1) = [0.01554;0.0025];
pgon(:, end+1) = [0.01595;0.00268];
pgon(:, end+1) = [0.01604;0.00268];
pgon(:, end+1) = [0.01645;0.0025];
pgon(:, end+1) = [0.0165;0.00245];
pgon(:, end+1) = [0.01668;0.00204];
pgon(:, end+1) = [0.01668;0.002];
pgon(:, end+1) = [0.02351;0.002];
pgon(:, end+1) = [0.02366;0.00166];
CSX = AddPolygon(CSX, 'PWR_copper', 200, 2, 0.0004, pgon);
% cutout
pgon = [];
pgon(:, end+1) = [0.004;0.004];
pgon(:, end+1) = [0.004;0.006];
pgon(:, end+1) = [0.006;0.006];
pgon(:, end+1) = [0.006;0.004];
CSX = AddPolygon(CSX, 'PWR_cutout', 201, 2, 0.0004, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.002;0.00151];
pgon(:, end+1) = [0.00234;0.00166];
pgon(:, end+1) = [0.00249;0.002];
pgon(:, end+1) = [0.00234;0.00234];
pgon(:, end+1) = [0.002;0.00249];
pgon(:, end+1) = [0.00166;0.00234];
pgon(:, end+1) = [0.00151;0.002];
pgon(:, end+1) = [0.00166;0.00166];
CSX = AddPolygon(CSX, 'PWR_copper', 200, 2, 0.0004, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.016;0.00151];
pgon(:, end+1) = [0.01634;0.00166];
pgon(:, end+1) = [0.01649;0.002];
pgon(:, end+1) = [0.01634;0.00234];
pgon(:, end+1) = [0.016;0.00249];
pgon(:, end+1) = [0.01566;0.00234];
pgon(:, end+1) = [0.01551;0.002];
pgon(:, end+1) = [0.01566;0.00166];
CSX = AddPolygon(CSX, 'PWR_copper', 200, 2, 0.0004, pgon);
% create layer Bottom material
CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper', 0, 3.5e-06);
% copper
pgon = [];
pgon(:, end+1) = [0.021;0.0146];
pgon(:, end+1) = [0.02128;0.01472];
pgon(:, end+1) = [0.0214;0.015];
pgon(:, end+1) = [0.02128;0.01528];
pgon(:, end+1) = [0.021;0.0154];
pgon(:, end+1) = [0.02072;0.01528];
pgon(:, end+1) = [0.0206;0.015];
pgon(:, end+1) = [0.02072;0.01472];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.002;0.0016];
pgon(:, end+1) = [0.00228;0.00172];
pgon(:, end+1) = [0.0024;0.002];
pgon(:, end+1) = [0.00228;0.00228];
pgon(:, end+1) = [0.002;0.0024];
pgon(:, end+1) = [0.00172;0.00228];
pgon(:, end+1) = [0.0016;0.002];
pgon(:, end+1) = [0.00172;0.00172];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.016;0.0016];
pgon(:, end+1) = [0.01628;0.00172];
pgon(:, end+1) = [0.0164;0.002];
pgon(:, end+1) = [0.01628;0.00228];
pgon(:, end+1) = [0.016;0.0024];
pgon(:, end+1) = [0.01572;0.00228];
pgon(:, end+1) = [0.0156;0.002];
pgon(:, end+1) = [0.01572;0.00172];
CSX = AddPolygon(CSX, 'Bottom_copper', 200, 2, 0, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.024;0.0016];
pgon(:, end+1) = [0.02428;0.00172];
pgon(:, end+1) = [0.0244;0.002];
pgon(:, end+1) = [0.02428;0.00228];
pgon(:, end+1) = [0.024;0.0024];
pgon(:, end+1) = [0.02372;0.00228];
pgon(:, end+1) = [0.0236;0.002];
pgon(:, end+1) = [0.02372;0.00172];
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
