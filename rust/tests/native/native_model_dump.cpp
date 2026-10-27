// Native CSXCAD model dump used by the hyp2mat RM-109 differential check.
//
// Reads every CSXCAD XML document given on the command line with the **native**
// `CSXCAD::ContinuousStructure::ReadFromXML` and prints a canonical model
// summary. `hyp2mat/rust/examples/model_dump.rs` prints the same summary from
// the Rust core, so the two can be diffed line by line.
//
// Build (see run_native_read_check.sh):
//   g++ -std=c++17 -O2 -o native_model_dump native_model_dump.cpp \
//       -I"$CSXCAD_INSTALL/include/CSXCAD" -I"$FPARSER/include" \
//       -L"$CSXCAD_BUILD/src" -Wl,-rpath,"$CSXCAD_BUILD/src" -lCSXCAD

#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

#include "CSBackgroundMaterial.h"
#include "CSPropMaterial.h"
#include "CSProperties.h"
#include "CSPrimitives.h"
#include "CSRectGrid.h"
#include "ContinuousStructure.h"

namespace {

void print_fixed(const char* label, double value) {
    std::printf("%s %.12f\n", label, value);
}

int dump(const char* path) {
    ContinuousStructure structure;
    const std::string error = structure.ReadFromXML(std::string(path));
    const char* name = std::strrchr(path, '/');
    name = name ? name + 1 : path;
    if (!error.empty()) {
        std::printf("file %s\nERROR %s\n", name, error.c_str());
        return 1;
    }

    std::printf("file %s\n", name);
    std::printf("properties %zu\n", structure.GetQtyProperties());
    std::printf("primitives %zu\n", structure.GetQtyPrimitives());

    CSRectGrid* grid = structure.GetGrid();
    if (!grid) {
        std::printf("grid 0 0 0\n");
        std::printf("delta_unit 0.000000000000\n");
    } else {
        std::printf("grid %zu %zu %zu\n", grid->GetQtyLines(0), grid->GetQtyLines(1),
                    grid->GetQtyLines(2));
        print_fixed("delta_unit", grid->GetDeltaUnit());
    }

    CSBackgroundMaterial* background = structure.GetBackgroundMaterial();
    if (background) {
        print_fixed("background_epsilon", background->GetEpsilon());
        print_fixed("background_mue", background->GetMue());
        print_fixed("background_kappa", background->GetKappa());
        print_fixed("background_sigma", background->GetSigma());
    }

    for (size_t index = 0; index < structure.GetQtyProperties(); ++index) {
        CSProperties* property = structure.GetProperty(index);
        if (!property) {
            std::printf("property <missing> <missing>\n");
            continue;
        }
        std::printf("property %s %s\n", property->GetTypeXMLString().c_str(),
                    property->GetName().c_str());
        if (property->GetType() == CSProperties::MATERIAL) {
            CSPropMaterial* material = dynamic_cast<CSPropMaterial*>(property);
            if (material) {
                print_fixed("property_epsilon", material->GetEpsilon(0));
                print_fixed("property_mue", material->GetMue(0));
                print_fixed("property_kappa", material->GetKappa(0));
                print_fixed("property_sigma", material->GetSigma(0));
            }
        }
    }

    std::vector<CSPrimitives*> primitives = structure.GetAllPrimitives(false);
    for (CSPrimitives* primitive : primitives) {
        if (!primitive) {
            std::printf("primitive <missing> 0 <missing>\n");
            continue;
        }
        CSProperties* owner = structure.HasPrimitive(primitive);
        std::printf("primitive %s %d %s\n", primitive->GetTypeName().c_str(),
                    primitive->GetPriority(),
                    owner ? owner->GetName().c_str() : "<orphan>");
    }
    return 0;
}

}  // namespace

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: %s <file.xml>...\n", argv[0]);
        return 2;
    }
    int status = 0;
    for (int argument = 1; argument < argc; ++argument) {
        if (dump(argv[argument]) != 0) {
            status = 1;
        }
    }
    return status;
}
