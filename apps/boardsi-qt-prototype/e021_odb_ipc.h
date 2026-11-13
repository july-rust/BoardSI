// E_021: strict, transactional ODB++ SOURCE-only Qt artwork transport.
// Separate from D_056_04 via-only IPC; never an electrical graph or SI result.
#pragma once
#include "d05605_canonical_artwork.h"
#include <QCryptographicHash>
#include <QDir>
#include <QDirIterator>
#include <QFile>
#include <QFileInfo>
#include <QPainterPathStroker>
#include <QSet>
#include <QStringList>
#include <cmath>

namespace e021ipc {
struct Source {
    d05605::Model artwork;
    QStringList layerNames;
};
inline bool mm(const QString &s, double &v, bool positive=false) {
    if (!d056ipc::rational(s,positive)) return false;
    v=d056ipc::dimension(s)*1000.;
    return std::isfinite(v) && (!positive || v>0) && std::abs(v)<=1000000.;
}
inline bool sourcePath(const QString &p) {
    return !p.isEmpty() && p.size()<=1024 && !p.startsWith('/') && !p.contains('\\')
        && !p.contains(':') && !p.contains(QChar::Null) &&
        std::all_of(p.split('/').cbegin(),p.split('/').cend(),[](const QString &s) {
            return !s.isEmpty() && s!="." && s!="..";
        });
}
inline bool decode(const QByteArray &wire,const QString &root,Source &out) {
    out={};
    if(wire.size()>4*1024*1024 || wire.isEmpty() || !wire.endsWith('\n')
       || wire.contains('\r') || !QFileInfo(root).isDir()
       || QFileInfo(root).isSymLink()) return false;
    const auto lines=QString::fromLatin1(wire).split('\n',Qt::SkipEmptyParts);
    if(lines.size()<6 || lines.size()>800) return false;
    const auto head=lines.front().split('|',Qt::KeepEmptyParts);
    int nf=0,nl=0,ng=0;
    if(head.size()!=7 || head[0]!="BOARDSI_ODB_QT_V1" ||
       !d056ipc::inRange(head[1],1,64,&nf) ||
       !d056ipc::inRange(head[2],2,64,&nl) ||
       !d056ipc::inRange(head[3],1,256,&ng) ||
       head[4]!="GEOMETRY_PARTIAL" || head[5]!="CONNECTIVITY_UNKNOWN" ||
       head[6]!="SI_NOT_RUN" || lines.size()!=nf+nl+ng+2) return false;
    Source next;
    next.artwork.layerCount=nl;
    QCryptographicHash manifest(QCryptographicHash::Sha256);
    QSet<QString> paths,ids;
    const QDir rootDir(root);
    for(int n=0;n<nf;++n) {
        const auto row=lines[n+1].split('|',Qt::KeepEmptyParts);
        if(row.size()!=3 || row[0]!="F" ||
           !d056ipc::hexToken(row[1],false) ||
           !d056ipc::hash64(row[2])) return false;
        const QByteArray raw=QByteArray::fromHex(row[1].toLatin1());
        if(QString::fromUtf8(raw).toUtf8()!=raw) return false;
        const QString p=QString::fromUtf8(raw);
        if(!sourcePath(p) || paths.contains(p)) return false;
        paths.insert(p);
        QString walk=rootDir.absolutePath();
        for (const QString &part:p.split('/')) {
            walk=QDir(walk).filePath(part);
            if(QFileInfo(walk).isSymLink()) return false;
        }
        const QString full=rootDir.filePath(p);
        QFile f(full);
        if(!QFileInfo(full).isFile() || !f.open(QIODevice::ReadOnly) ||
           f.size()<0 || f.size()>16*1024*1024) return false;
        QCryptographicHash hash(QCryptographicHash::Sha256);
        while(!f.atEnd()) {
            QByteArray block=f.read(65536);
            if(block.isEmpty()) return false;
            hash.addData(block);
        }
        if(QString::fromLatin1(hash.result().toHex())!=row[2]) return false;
        manifest.addData(row[1].toLatin1());
        manifest.addData("|",1);
        manifest.addData(row[2].toLatin1());
        manifest.addData("\n",1);
    }
    // A SOURCE file omitted by a stale IPC invalidates the entire snapshot.
    QDirIterator it(root,QDir::AllEntries|QDir::Hidden|QDir::System|QDir::NoDotAndDotDot,
                    QDirIterator::Subdirectories);
    int seen=0;
    while(it.hasNext()) {
        const QString full=it.next();
        const QFileInfo info(full);
        if(info.isSymLink()) return false;
        if(info.isFile()) {
            const QString rel=rootDir.relativeFilePath(full).replace('\\','/');
            if(!paths.contains(rel) || ++seen>nf) return false;
        } else if (!info.isDir()) return false;
    }
    if(seen!=nf) return false;
    next.artwork.sourceSha=QString::fromLatin1(manifest.result().toHex());
    for(int n=0;n<nl;++n) {
        const auto row=lines[1+nf+n].split('|',Qt::KeepEmptyParts);
        int i=0;
        if(row.size()!=4 || row[0]!="L" ||
           !d056ipc::inRange(row[1],1,nl,&i) || i!=n+1 ||
           !d056ipc::hexToken(row[2],false) || !d056ipc::hexToken(row[3],true))
            return false;
        const QByteArray name=QByteArray::fromHex(row[2].toLatin1());
        if(name.isEmpty() || name.size()>128) return false;
        next.layerNames.append(QString::fromUtf8(name));
    }
    bool bounds=false;
    for(int n=0;n<ng;++n) {
        const auto row=lines[1+nf+nl+n].split('|',Qt::KeepEmptyParts);
        if(row.size()<2 || (row[0]!="T" && row[0]!="P")) return false;
        const bool trace=row[0]=="T";
        if(row.size()!=(trace?9:10)) return false;
        int layer=0;
        if(!d056ipc::inRange(row[1],1,nl,&layer) ||
           !d056ipc::hexToken(row[2],true) || ids.contains(row[2]))
            return false;
        ids.insert(row[2]);
        d05605::Glyph g;
        g.layer=layer; g.conductive=true; g.netId=row[trace?8:8];
        if(g.netId!="-" && !d056ipc::hexToken(g.netId,true))return false;
        g.sourceSha=next.artwork.sourceSha;
        g.shape=trace?"TRACE":"PAD";
        double x=0,y=0,a=0,b=0,c=0;
        if(!mm(row[3],x) || !mm(row[4],y)) return false;
        QPainterPath path;
        if(trace) {
            if(!mm(row[5],a) || !mm(row[6],b) || !mm(row[7],c,true) ||
               c>1000 || (a==x && b==y)) return false;
            QPainterPath centre;
            centre.moveTo(x,-y); centre.lineTo(a,-b);
            QPainterPathStroker stroke;
            stroke.setWidth(c);stroke.setCapStyle(Qt::RoundCap);stroke.setJoinStyle(Qt::RoundJoin);
            path=stroke.createStroke(centre);
        } else {
            if(!d056ipc::hexToken(row[9],true) ||
               (row[5]!="CIRCLE" && row[5]!="RECT") ||
               !mm(row[6],a,true) || !mm(row[7],b,row[5]=="RECT") ||
               a>1000 || b>1000) return false;
            g.shape=row[5];
            if(row[5]=="CIRCLE") {
                if(row[7]!="0:1")return false;
                path.addEllipse(QPointF(x,-y),a/2,a/2);
            } else path.addRect(QRectF(x-a/2,-y-b/2,a,b));
        }
        if(path.isEmpty() || path.boundingRect().isEmpty())return false;
        g.footprint=path; g.outline=path;
        g.details=QString("ODB++ SOURCE %1 / object %2 / physical L%3 / SOURCE manifest SHA256 %4"
                          " / DESIGN net ownership %5 / manufacture and connectivity UNKNOWN / SI NOT_RUN")
                  .arg(trace?"TRACE":"PAD",d05605::readableId(row[2]))
                  .arg(layer).arg(g.sourceSha).arg(g.netId=="-"?"UNKNOWN":d05605::readableId(g.netId));
        const QRectF rect=path.boundingRect();
        if(!bounds) {next.artwork.bounds=rect;bounds=true;}
        else next.artwork.bounds=next.artwork.bounds.united(rect);
        next.artwork.glyphs.append(g);
    }
    const auto end=lines.back().split('|',Qt::KeepEmptyParts);
    int count=0;
    if(!bounds || end.size()!=3 || end[0]!="END" ||
       !d056ipc::inRange(end[1],1,256,&count) || count!=ng ||
       end[2]!="SOURCE_ONLY")return false;
    out=std::move(next);
    return true;
}
} // namespace e021ipc
