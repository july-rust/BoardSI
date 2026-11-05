// D_039 independent native Qt local copper geometry, not a physical SI solver.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d039_local_copper_qt.h"
#include <QApplication>

int main(int argc, char **argv) {
    QApplication app(argc, argv);
    if (argc != 3) return 2;
    return d039::selftest(QString::fromLocal8Bit(argv[1]),
                          QString::fromLocal8Bit(argv[2]));
}
