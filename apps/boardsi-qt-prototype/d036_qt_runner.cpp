// D_036 independent native Qt SOURCE graph verification; never an SI solver.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d036_dual_mirrored_via_smoke.h"
#include <QApplication>

int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if (argc!=2) return 2;
    return d036::selftest(QString::fromLocal8Bit(argv[1]));
}
