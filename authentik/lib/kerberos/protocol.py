from enum import UNIQUE, Enum, verify

from asn1crypto import core as asn1


@verify(UNIQUE)
class MessageType(Enum):
    """
    Kerberos message types as defined by RFC 4120.

    See https://www.rfc-editor.org/rfc/rfc4120#section-7.5.7
    """

    KRB_AS_REQ = 10  #  Request for initial authentication
    KRB_AS_REP = 11  # Response to KRB_AS_REQ request
    KRB_TGS_REQ = 12  # Request for authentication based on TGT
    KRB_TGS_REP = 13  # Response to KRB_TGS_REQ request
    KRB_AP_REQ = 14  # Application request to server
    KRB_AP_REP = 15  # Response to KRB_AP_REQ_MUTUAL
    KRB_RESERVED16 = 16  # Reserved for user-to-user krb_tgt_request
    KRB_RESERVED17 = 17  # Reserved for user-to-user krb_tgt_reply
    KRB_SAFE = 20  # Safe (checksummed) application message
    KRB_PRIV = 21  # Private (encrypted) application message
    KRB_CRED = 22  # Private (encrypted) message to forward credentials
    KRB_ERROR = 30  # Error response


class GeneralStringSequence(asn1.SequenceOf):
    _child_spec = asn1.GeneralString


class PreAuthenticationData(asn1.Sequence):
    _fields = [
        ("padata-type", asn1.Integer, {"tag_type": "explicit", "tag": 1}),
        ("padata-value", asn1.OctetString, {"tag_type": "explicit", "tag": 2}),
    ]


class PreAuthenticationDataSequence(asn1.SequenceOf):
    _child_spec = PreAuthenticationData


class KdcOptions(asn1.BitString):
    _map = {
        0: "reserved",
        1: "forwardable",
        2: "forwarded",
        3: "proxiable",
        4: "proxy",
        5: "allow-postdate",
        6: "postdated",
        7: "unused7",
        8: "renewable",
        9: "unused9",
        10: "unused10",
        11: "opt-hardware-auth",
        12: "unused12",
        13: "unused13",
        14: "constrained-delegation",
        15: "canonicalize",
        16: "request-anonymous",
        17: "unused17",
        18: "unused18",
        19: "unused19",
        20: "unused20",
        21: "unused21",
        22: "unused22",
        23: "unused23",
        24: "unused24",
        25: "unused25",
        26: "disable-transited-check",
        27: "renewable-ok",
        28: "enc-tkt-in-skey",
        30: "renew",
        31: "validate",
    }


class PrincipalName(asn1.Sequence):
    _fields = [
        ("name-type", asn1.Integer, {"tag_type": "explicit", "tag": 0}),
        ("name-string", GeneralStringSequence, {"tag_type": "explicit", "tag": 1}),
    ]


class EncryptionTypeSequence(asn1.Sequence):
    _child_spec = asn1.Integer


class HostAddress(asn1.Sequence):
    _fields = [
        ("addr-type", asn1.Integer, {"tag_type": "explicit", "tag": 0}),
        ("address", asn1.OctetString, {"tag_type": "explicit", "tag": 1}),
    ]


class HostAddressSequence(asn1.SequenceOf):
    _child_spec = HostAddress


class EncryptedData(asn1.Sequence):
    _fields = [
        ("etype", asn1.Integer, {"tag_type": "explicit", "tag": 0}),
        ("kvno", asn1.Integer, {"tag_type": "explicit", "tag": 1, "optional": True}),
        ("cipher", asn1.OctetString, {"tag_type": "explicit", "tag": 2}),
    ]


class Ticket(asn1.Sequence):
    explicit = (asn1.CLASS_NAME_TO_NUM_MAP["application"], 1)

    _fields = [
        ("tkt-vno", asn1.Integer, {"tag_type": "explicit", "tag": 0}),
        ("realm", asn1.GeneralString, {"tag_type": "explicit", "tag": 1}),
        ("sname", PrincipalName, {"tag_type": "explicit", "tag": 2}),
        ("enc-part", EncryptedData, {"tag_type": "explicit", "tag": 3}),
    ]


class TicketSequence(asn1.SequenceOf):
    _child_spec = Ticket


class KdcReqBody(asn1.Sequence):
    _fields = [
        ("kdc-options", KdcOptions, {"tag_type": "explicit", "tag": 0}),
        ("cname", PrincipalName, {"tag_type": "explicit", "tag": 1, "optional": True}),
        ("realm", asn1.GeneralString, {"tag_type": "explicit", "tag": 2}),
        ("sname", PrincipalName, {"tag_type": "explicit", "tag": 3, "optional": True}),
        ("from", asn1.GeneralizedTime, {"tag_type": "explicit", "tag": 4, "optional": True}),
        ("till", asn1.GeneralizedTime, {"tag_type": "explicit", "tag": 5, "optional": True}),
        ("rtime", asn1.GeneralizedTime, {"tag_type": "explicit", "tag": 6, "optional": True}),
        ("nonce", asn1.Integer, {"tag_type": "explicit", "tag": 7}),
        ("etype", EncryptionTypeSequence, {"tag_type": "explicit", "tag": 8}),
        ("addresses", HostAddressSequence, {"tag_type": "explicit", "tag": 9, "optional": True}),
        (
            "enc-authorization-data",
            EncryptedData,
            {"tag_type": "explicit", "tag": 10, "optional": True},
        ),
        (
            "additional-tickets",
            TicketSequence,
            {"tag_type": "explicit", "tag": 11, "optional": True},
        ),
    ]


class KdcReq(asn1.Sequence):
    _fields = [
        ("pvno", asn1.Integer, {"tag_type": "explicit", "tag": 1}),
        ("msg-type", asn1.Integer, {"tag_type": "explicit", "tag": 2}),
        (
            "pa-data",
            PreAuthenticationDataSequence,
            {"tag_type": "explicit", "tag": 3, "optional": True},
        ),
        ("req-body", KdcReqBody, {"tag_type": "explicit", "tag": 4}),
    ]


class AsReq(KdcReq):
    explicit = (asn1.CLASS_NAME_TO_NUM_MAP["application"], 10)


class TgsReq(KdcReq):
    explicit = (asn1.CLASS_NAME_TO_NUM_MAP["application"], 12)
