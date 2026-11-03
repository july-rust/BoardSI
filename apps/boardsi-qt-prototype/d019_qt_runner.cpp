// D_019 independent native Qt Source-only T witness; not a physical solver.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d019_oblique_t_junction_smoke.h"
#include <QApplication>
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if (argc!=2) return 2;
    return d019::selftest(QString::fromLocal8Bit(argv[1]));
}
