// D_034 standalone native Qt importer revalidation; Source candidate, not solver.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d034_non45_oblique_mirrored_via_b_endpoint_smoke.h"
#include <QApplication>
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if (argc!=2) return 2;
    return d034::selftest(QString::fromLocal8Bit(argv[1]));
}
