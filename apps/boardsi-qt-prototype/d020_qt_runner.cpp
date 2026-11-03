// D_020 independent native Qt contained collinear Source witness; never a physical solver.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d020_contained_overlap_smoke.h"
#include <QApplication>
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if (argc!=2) return 2;
    return d020::selftest(QString::fromLocal8Bit(argv[1]));
}
