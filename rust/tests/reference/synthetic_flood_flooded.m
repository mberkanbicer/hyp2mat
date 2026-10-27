function CSX = pcb(CSX)
% matlab script created by hyp2mat
% create minimal mesh
mesh = {};
mesh.x = [0 0.02];
mesh.y = [0 0.01];
mesh.z = [0 0.0015];
% add mesh
CSX = DefineRectGrid(CSX, 1, mesh);
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_DL02', 4.5, 0.01);
% board outline, layer DL02
CSX = AddBox(CSX, 'Dielectric_DL02', 100, [ 0, 0, 0], [ 0.02, 0.01, 0.0005]  );
CSX = AddHyperLynxDielectric(CSX, 'Dielectric_DL01', 4.5, 0.01);
% board outline, layer DL01
CSX = AddBox(CSX, 'Dielectric_DL01', 100, [ 0, 0, 0.0005], [ 0.02, 0.01, 0.0015]  );
% copper
% create layer Top material
CSX = AddHyperLynxMetal2D(CSX, 'Top_copper', 0, 3.5e-06);
% create layer Top cutout
CSX = AddMaterial( CSX, 'Top_cutout');
CSX = SetMaterialProperty( CSX, 'Top_cutout', 'Epsilon', 1, 'Mue', 1);
% copper
pgon = [];
pgon(:, end+1) = [0.02;0];
pgon(:, end+1) = [0.02;0.01];
pgon(:, end+1) = [0;0.01];
pgon(:, end+1) = [0;0];
CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.0015, pgon);
% cutout
pgon = [];
pgon(:, end+1) = [0.00197;0.0047];
pgon(:, end+1) = [0.00193;0.00471];
pgon(:, end+1) = [0.00181;0.00476];
pgon(:, end+1) = [0.00176;0.00481];
pgon(:, end+1) = [0.0017;0.00495];
pgon(:, end+1) = [0.0017;0.00504];
pgon(:, end+1) = [0.00176;0.00518];
pgon(:, end+1) = [0.00181;0.00523];
pgon(:, end+1) = [0.00193;0.00528];
pgon(:, end+1) = [0.00197;0.0053];
pgon(:, end+1) = [0.00802;0.0053];
pgon(:, end+1) = [0.00806;0.00528];
pgon(:, end+1) = [0.00818;0.00523];
pgon(:, end+1) = [0.00823;0.00518];
pgon(:, end+1) = [0.00829;0.00504];
pgon(:, end+1) = [0.00829;0.00495];
pgon(:, end+1) = [0.00823;0.00481];
pgon(:, end+1) = [0.00818;0.00476];
pgon(:, end+1) = [0.00806;0.00471];
pgon(:, end+1) = [0.00802;0.0047];
CSX = AddPolygon(CSX, 'Top_cutout', 201, 2, 0.0015, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.008;0.0049];
pgon(:, end+1) = [0.00807;0.00493];
pgon(:, end+1) = [0.0081;0.005];
pgon(:, end+1) = [0.00807;0.00507];
pgon(:, end+1) = [0.008;0.0051];
pgon(:, end+1) = [0.002;0.0051];
pgon(:, end+1) = [0.00193;0.00507];
pgon(:, end+1) = [0.0019;0.005];
pgon(:, end+1) = [0.00193;0.00493];
pgon(:, end+1) = [0.002;0.0049];
CSX = AddPolygon(CSX, 'Top_copper', 202, 2, 0.0015, pgon);
% create layer GND material
CSX = AddHyperLynxMetal2D(CSX, 'GND_copper', 0, 3.5e-06);
% create layer GND cutout
CSX = AddMaterial( CSX, 'GND_cutout');
CSX = SetMaterialProperty( CSX, 'GND_cutout', 'Epsilon', 4.5, 'Mue', 1);
% copper
pgon = [];
pgon(:, end+1) = [0.02;0];
pgon(:, end+1) = [0.02;0.01];
pgon(:, end+1) = [0;0.01];
pgon(:, end+1) = [0;0];
CSX = AddPolygon(CSX, 'GND_copper', 200, 2, 0.0005, pgon);
% cutout
pgon = [];
pgon(:, end+1) = [0.00091;0.0008];
pgon(:, end+1) = [0.0008;0.00091];
pgon(:, end+1) = [0.0008;0.00908];
pgon(:, end+1) = [0.00091;0.0092];
pgon(:, end+1) = [0.01908;0.0092];
pgon(:, end+1) = [0.0192;0.00908];
pgon(:, end+1) = [0.0192;0.00091];
pgon(:, end+1) = [0.01908;0.0008];
CSX = AddPolygon(CSX, 'GND_cutout', 201, 2, 0.0005, pgon);
% copper
pgon = [];
pgon(:, end+1) = [0.019;0.001];
pgon(:, end+1) = [0.019;0.009];
pgon(:, end+1) = [0.001;0.009];
pgon(:, end+1) = [0.001;0.001];
CSX = AddPolygon(CSX, 'GND_copper', 202, 2, 0.0005, pgon);
% devices
CSX.HyperLynxDevice = {};
% ports
CSX.HyperLynxPort = {};
%not truncated
