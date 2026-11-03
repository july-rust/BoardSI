// D_016 stand-alone native Qt Source geometry witness, not a solver.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d016_x_junction_smoke.h"
#include <QApplication>
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if (argc!=2) return 2;
    return d016::selftest(QString::fromLocal8Bit(argv[1]));
}
