// D_037 independent native Qt SOURCE graph verification; never an SI solver.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d037_mixed_sign_dual_via_smoke.h"
#include <QApplication>

int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if (argc!=2) return 2;
    return d037::selftest(QString::fromLocal8Bit(argv[1]));
}
