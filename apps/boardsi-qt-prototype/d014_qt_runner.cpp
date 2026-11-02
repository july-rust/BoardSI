// D_014 standalone independent Qt importer/Source verifier, not a solver.
#include "preview_import.h"
#include "d014_two_via_smoke.h"
#include <QApplication>
int main(int argc,char **argv){
    QApplication app(argc,argv);
    if(argc!=2)return 2;
    return d014::selftest(QString::fromLocal8Bit(argv[1]));
}
