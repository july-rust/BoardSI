// A_020 P2-K: independent Qt imported-artwork explorer regression.
// Offline synthetic data only; no electrical connectivity, full PADS, or SI claims.
#pragma once
#include <QCheckBox>
#include <QComboBox>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLineEdit>
#include <QPushButton>
#include <QGraphicsScene>
#include <QFile>
#include <QSysInfo>

static int runA020ExplorerSmoke() {
    ImportedArtworkPanel panel;
    QString details;
    panel.setDetailsCallback([&details](const QString &value) { details=value; });

    ImportedTrack top;
    top.layer=1; top.net="NET_A"; top.x0_nm=0; top.y0_nm=0;
    top.x1_nm=2540000; top.y1_nm=0; top.width_nm=25400;
    ImportedTrack bottom=top;
    bottom.layer=4; bottom.net="NET_B";
    bottom.y0_nm=2540000; bottom.y1_nm=2540000;
    ImportedVia via;
    via.name="VIA_A"; via.net="NET_A"; via.firstLayer=1; via.lastLayer=4;
    via.x_nm=0; via.y_nm=0; via.diameter_nm=600000; via.drill_nm=250000;
    ImportedPad pad;
    pad.reference="U1"; pad.pin="1"; pad.shape="R";
    pad.layer=1; pad.side="T"; pad.width_nm=400000; pad.height_nm=400000;
    pad.x_nm=0; pad.y_nm=0;

    std::vector<ImportedOutlineEdge> outline;
    const qint64 xy[5][2]={{-2540000,-2540000},{5080000,-2540000},
                           {5080000,5080000},{-2540000,5080000},
                           {-2540000,-2540000}};
    for(int i=0;i<4;++i) {
        ImportedOutlineEdge e;
        e.name="OUTER"; e.sequence=i;
        e.x0_nm=xy[i][0]; e.y0_nm=xy[i][1];
        e.x1_nm=xy[i+1][0]; e.y1_nm=xy[i+1][1];
        outline.push_back(e);
    }
    bool imported=panel.load(4,0,{top,bottom},{via},{pad},{},outline);
    auto *layers=panel.findChild<QComboBox *>("importedLayerSelect");
    auto *solo=panel.findChild<QPushButton *>("importedSoloLayer");
    auto *restore=panel.findChild<QPushButton *>("importedRestoreLayers");
    auto *hide=panel.findChild<QPushButton *>("importedHideAllLayers");
    auto *show=panel.findChild<QPushButton *>("importedShowAllLayers");
    auto *search=panel.findChild<QLineEdit *>("importedNetSearch");
    auto *find=panel.findChild<QPushButton *>("importedNetFind");
    auto *netOnly=panel.findChild<QCheckBox *>("importedNetOnly");
    const bool controls=layers && solo && restore && hide && show &&
                        search && find && netOnly;
    bool layersPass=false, searchPass=false, netOnlyPass=false;
    bool propertiesPass=false, resetPass=false, malformedRejected=false;
    bool outlineSelected=false, caseIdentity=false;
    bool rhiChecked=false, rhiNetOnlyParity=true;

    if(imported && controls) {
        layers->setCurrentIndex(layers->findData(1));
        const bool initial=panel.visibleSegments()==2;
        solo->click();
        const bool soloPass=panel.visibleSegments()==1;
        restore->click();
        const bool restorePass=panel.visibleSegments()==2;
        hide->click();
        const bool hidden=panel.visibleSegments()==0;
        show->click();
        layersPass=initial && soloPass && restorePass && hidden &&
                   panel.visibleSegments()==2;

        search->setText("net_b");
        find->click();
        const bool matched=details.contains("Selected net: NET_B");
        search->setText("NOT_A_NET");
        find->click();
        const bool unknownRejected=details.contains("Selected net: NET_B");
        searchPass=matched && unknownRejected && panel.selectNet("NET_A");

        netOnly->setChecked(true);
        bool padHidden=false, viaVisible=false;
        auto *scene=panel.findChild<QGraphicsScene *>();
        if(scene) for(auto *item:scene->items()) {
            if(item->data(1).toString().startsWith("Pad U1.1"))
                padHidden = !item->isVisible();
            if(item->data(1).toString().startsWith("Via VIA_A"))
                viaVisible = item->isVisible();
        }
        netOnlyPass=panel.visibleSegments()==1 && padHidden && viaVisible &&
                    details.contains("Show selected net only: YES") &&
                    details.contains("Net connectivity: NOT_VERIFIED");

#ifdef BOARDSI_RHI_ARTWORK
        if(auto *rhi=panel.rhiViewForTest();rhi && panel.rhiCanvasActiveForTest()) {
            rhiChecked=true;
            rhiNetOnlyParity=rhi->itemForTest()->visiblePadCount()==0;
        }
#endif

        const bool trace=panel.selectTrack(0) &&
            details.contains("start (0.000000, 0.000000) mm") &&
            details.contains("length 2.540000 mm") &&
            details.contains("connectivity NOT_VERIFIED");
        const bool viaDetail=panel.selectVia(0) &&
            details.contains("Via VIA_A") &&
            details.contains("centre (0.000000, 0.000000) mm");
        netOnly->setChecked(false);
        const bool padDetail=panel.selectPad(0) &&
            details.contains("Pad U1.1") &&
            details.contains("net NOT_VERIFIED");
        const bool outlineDetail=panel.selectOutline(0) &&
            details.contains("Board Outline OUTER") &&
            details.contains("segment 0") &&
            details.contains("not copper");
        outlineSelected=outlineDetail;
        propertiesPass=trace && viaDetail && padDetail && outlineDetail;

        ImportedTrack broken=top;
        broken.width_nm=0;
        malformedRejected=!panel.load(4,0,{broken}) &&
                          panel.visibleSegments()==2 &&
                          panel.segmentCount()==2;

        panel.clearArtwork();
        resetPass=panel.segmentCount()==0 &&
                  panel.visibleSegments()==0 &&
                  !netOnly->isChecked() &&
                  panel.findChild<QComboBox *>("importedNetSelect")->count()==1;
        ImportedTrack lower=top;
        lower.net="net_a"; // Source labels can differ only by ASCII case.
        if(panel.load(4,0,{top,lower})) {
            search->setText("net_a"); find->click();
            const bool exact=details.contains("Selected net: net_a");
            search->setText("NeT_A"); find->click();
            caseIdentity=exact && details.contains("Selected net: net_a") &&
                         panel.selectNet("NET_A") &&
                         details.contains("Selected net: NET_A");
        }
    }
    const bool pass=imported && controls && layersPass && searchPass &&
                    netOnlyPass && propertiesPass && resetPass &&
                    malformedRejected && outlineSelected && caseIdentity && rhiNetOnlyParity;
    const QJsonObject report{
        {"schema","boardsi-preview-a020-explorer-v1"},
        {"pass",pass},
        {"synthetic_import",imported},
        {"controls_present",controls},
        {"layer_solo_hide_restore",layersPass},
        {"case_insensitive_net_search",searchPass},
        {"net_only_unverified_pad_excluded",netOnlyPass},
        {"source_properties",propertiesPass},
        {"straight_outline_selection",outlineSelected},
        {"case_sensitive_exact_net_identity",caseIdentity},
        {"malformed_import_preserves_source",malformedRejected},
        {"clear_resets_filter_and_model",resetPass},
        {"rhi_source_checked",rhiChecked},
        {"rhi_visibility_parity",rhiNetOnlyParity},
        {"physical_si","NOT_RUN"},
        {"gpu_fps","NOT_MEASURED"},
        {"gpu_vram","NOT_MEASURED"},
        {"source_sha",qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"native_platform",QSysInfo::currentCpuArchitecture()}
    };
    QFile receipt("qt-preview-a020-explorer-results.json");
    if(!receipt.open(QIODevice::WriteOnly) ||
       receipt.write(QJsonDocument(report).toJson(QJsonDocument::Indented))<1)
        return 2;
    qInfo().noquote()<<QJsonDocument(report).toJson(QJsonDocument::Compact);
    return pass?0:1;
}
