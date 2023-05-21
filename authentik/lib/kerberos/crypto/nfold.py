import math


def nfold(data: bytes, size: int) -> bytes:
    """
    Streches data to be of length `size`. Size must be in bytes.

    See https://www.rfc-editor.org/rfc/rfc3961#section-5.1

    Implementation example in https://www.gnu.org/software/shishi/ides.pdf
    """

    def rot13(data: bytes, step: int) -> bytes:
        """
        Rotate `data` 13 bits right. Step is the number of times the rotation will be performed.
        """

        if step == 0:
            return data

        _data = int.from_bytes(data, byteorder="big")
        mod = (1 << (step * 8)) - 1

        if step == 1:
            shift = 5
        else:
            shift = 13
        result = ((_data >> shift) | (_data << (step * 8 - shift))) & mod
        return result.to_bytes(step, byteorder="big")

    def ones_complement_add(lhs: bytes, rhs: bytes) -> bytes:
        """
        One's complement addition (with end-around carry).
        """
        if len(lhs) != len(rhs):
            raise ValueError("Cannot one's complement add two numbers of different size")
        size = len(lhs)

        result = [l + r for l, r in zip(lhs, rhs)]
        while any(l & ~0xFF for l in result):
            result = [(result[i - size + 1] >> 8) + (result[i] & 0xFF) for i in range(size)]
        return bytes(result)

    # def ones_complement_add(add1: bytes, add2: bytes) -> bytes:
    #     """
    #     One's complement addition (without end-around carry).
    #     """
    #     if len(add1) != len(add2):
    #         raise ValueError("Cannot one's complement add two numbers of different size")
    #     size = len(add1)
    #     mod = 1 << (size * 8)
    #     result = int.from_bytes(add1, byteorder="big") + int.from_bytes(add2, byteorder="big")
    #     return (
    #         result.to_bytes(size, byteorder="big")
    #         if result < mod
    #         else ((result + 1) % mod).to_bytes(size, byteorder="big")
    #     )

    data_size = len(data)
    lcm = math.lcm(size, data_size)

    buf = bytes()
    for _ in range(lcm // data_size):
        buf += data
        data = rot13(data, data_size)

    result = bytes([0] * size)
    for i in range(0, lcm, size):
        result = ones_complement_add(result, buf[i : i + size])

    return result
