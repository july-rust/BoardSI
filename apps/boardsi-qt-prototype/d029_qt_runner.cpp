// D_029 independent Qt importer verification; geometry candidate only.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d029_non45_oblique_via_a_endpoint_smoke.h"
#include <QApplication>
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if (argc!=2) return 2;
    return d029::selftest(QString::fromLocal8Bit(argv[1]));
}
