// D_040 independent native Qt imported-source component isolation.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d040_multi_component_qt.h"
#include <QApplication>

int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if(argc!=3)return 2;
    return d040::selftest(QString::fromLocal8Bit(argv[1]),
                          QString::fromLocal8Bit(argv[2]));
}
