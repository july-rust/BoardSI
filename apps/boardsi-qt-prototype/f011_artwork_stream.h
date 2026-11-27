// F_011: bounded NDJSON receiver for one native QProcess request.
// All frames are private until an exact count and SHA-256 terminal receipt.
#pragma once
#include <QByteArray>
#include <QCryptographicHash>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QString>

class F011ArtworkStream final {
    static constexpr qsizetype MaxFrame=60000;
    static constexpr qsizetype MaxTotal=16*1024*1024;
    static constexpr int MaxItems=10000;
    static constexpr int MaxBatchItems=128;
    QByteArray pending_;
    QCryptographicHash hash_{QCryptographicHash::Sha256};
    QJsonObject receipt_;
    QJsonArray rows_;
    QString token_;
    qsizetype bytes_=0;
    int expected_=0, sequence_=0;
    bool began_=false, ended_=false, failed_=false;

    bool reject() {failed_=true;return false;}
    static bool integer(const QJsonValue &value,int expected) {
        return value.isDouble() && value.toDouble()==double(expected);
    }
    bool frame(const QByteArray &line) {
        if(ended_ || line.size()>=MaxFrame || line.isEmpty())return reject();
        QJsonParseError error;
        const auto document=QJsonDocument::fromJson(line,&error);
        if(error.error!=QJsonParseError::NoError || !document.isObject())return reject();
        const auto obj=document.object();
        if(obj.value("protocol").toString()!="BSI_ARTWORK_STREAM_V1" ||
           obj.value("request").toString()!=token_)return reject();
        const auto kind=obj.value("kind").toString();
        if(!began_) {
            if(kind!="begin" || !obj.value("receipt").isObject())return reject();
            const auto count=obj.value("item_count");
            const auto n=count.toInt(-1);
            if(n<0 || n>MaxItems || !integer(count,n))return reject();
            receipt_=obj.value("receipt").toObject();
            const bool renderable=receipt_.value("artwork_status").toString()=="RENDERABLE";
            if(renderable!=(n>0) ||
               (renderable && (!receipt_.value("artwork").isObject() ||
                receipt_.value("artwork").toObject().contains("items"))) ||
               (!renderable && receipt_.contains("artwork")))return reject();
            expected_=n;began_=true;
            hash_.addData(line);
            return true;
        }
        if(kind=="batch") {
            if(!integer(obj.value("sequence"),sequence_) ||
               !obj.value("items").isArray())return reject();
            const auto items=obj.value("items").toArray();
            if(items.isEmpty() || items.size()>MaxBatchItems ||
               items.size()>expected_-rows_.size())return reject();
            for(const auto &row:items) {
                if(!row.isObject())return reject();
                rows_.append(row);
            }
            ++sequence_;
            hash_.addData(line);
            return true;
        }
        if(kind!="end" || !integer(obj.value("batch_count"),sequence_) ||
           !integer(obj.value("item_count"),expected_) || rows_.size()!=expected_ ||
           obj.value("sha256").toString()!=QString::fromLatin1(hash_.result().toHex()))
            return reject();
        ended_=true;
        return true;
    }
public:
    void reset(const QString &token) {
        pending_.clear();hash_.reset();receipt_=QJsonObject();rows_=QJsonArray();
        token_=token;bytes_=0;expected_=0;sequence_=0;
        began_=false;ended_=false;failed_=false;
    }
    bool feed(const QByteArray &chunk) {
        if(failed_ || chunk.size()>MaxTotal-bytes_)return reject();
        bytes_+=chunk.size();
        qsizetype offset=0;
        while(offset<chunk.size()) {
            const auto newline=chunk.indexOf('\n',offset);
            const auto size=(newline<0 ? chunk.size() : newline+1)-offset;
            if(size>=MaxFrame-pending_.size())return reject();
            pending_.append(chunk.constData()+offset,size);
            offset+=size;
            if(newline<0)break;
            if(!frame(pending_))return false;
            pending_.clear();
        }
        return true;
    }
    bool finish(QJsonObject &receipt) {
        if(failed_ || !began_ || !ended_ || !pending_.isEmpty())return reject();
        receipt=receipt_;
        if(expected_>0) {
            auto artwork=receipt.value("artwork").toObject();
            artwork.insert("items",rows_);
            receipt.insert("artwork",artwork);
        }
        return true;
    }
};
