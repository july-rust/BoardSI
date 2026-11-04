// D_027 standalone native Qt importer revalidation; Source candidate, not solver.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d027_oblique_via_a_endpoint_smoke.h"
#include <QApplication>
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if (argc!=2) return 2;
    return d027::selftest(QString::fromLocal8Bit(argv[1]));
}
