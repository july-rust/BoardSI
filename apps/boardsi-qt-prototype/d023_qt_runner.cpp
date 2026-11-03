// D_023 standalone native Qt importer revalidation; source-only, no physical solver.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d023_opposite_endpoint_smoke.h"
#include <QApplication>
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if (argc!=2) return 2;
    return d023::selftest(QString::fromLocal8Bit(argv[1]));
}
