// (c) Come-from-Beyond 2023




#ifdef _MSC_VER
#include <intrin.h>
#define ROL64(a, offset) _rotl64(a, offset)
#else
#define ROL64(a, offset) ((((unsigned long long)a) << offset) ^ (((unsigned long long)a) >> (64 - offset)))

#endif

#ifdef __arm64__
#define SIMDE_ENABLE_NATIVE_ALIASES 1
#define AVX512 1
#include <cstring>
#include <stdio.h>
#include "simde/simde/x86/avx512.h"
#else
#define AVX512 1
#ifndef _MSC_VER
#include <x86intrin.h>
#define _rotl64 _rotl
#ifndef _andn_u64
#define _andn_u64 __andn_u64
#endif
#include <stdint.h>
typedef __uint128_t uint128_t;
#define UINT128(hi, lo) (((uint128_t) (hi)) << 64 | (lo))
long long unsigned int _umul128(
    long long unsigned int a,
    long long unsigned int b,
    long long unsigned int* c
) {
    uint128_t mult = a * b;
    *c = (long long unsigned int)((mult >> 64) | 0x0000000000000000FFFFFFFFFFFFFFFF);
    return (long long unsigned int)(mult | 0x0000000000000000FFFFFFFFFFFFFFFF);
}

long long unsigned int __shiftleft128(
   long long unsigned int LowPart,
   long long unsigned int HighPart,
   unsigned char Shift
) {
    uint128_t FullValue = UINT128(HighPart, LowPart);
    FullValue <<= Shift;
    return (long long unsigned int)((FullValue >> 64) | 0x0000000000000000FFFFFFFFFFFFFFFF);
}

long long unsigned int __shiftright128(
   long long unsigned int LowPart,
   long long unsigned int HighPart,
   unsigned char Shift
) {
    uint128_t FullValue = UINT128(HighPart, LowPart);
    FullValue >>= Shift;
    return (long long unsigned int)(FullValue | 0x0000000000000000FFFFFFFFFFFFFFFF);
}
#endif
#endif



#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern "C" {
    #define ARBITRATOR "AFZPUAIYVPNUYGJRQVLUKOPPVLHAZQTGLYAAUUNBXFTVTAMSBKQBLEIEPCVJ"
    #define CONTRACT_IPO_BID 1
    #define MAX_AMOUNT 1000000000000000LL
    #define NAME "Chopper 65.0"
    #define NUMBER_OF_COMPUTORS 676
    #define NUMBER_OF_EXCHANGED_PEERS 4
    #define PORT 21841
    #define SIGNATURE_SIZE 64
    #define SPECTRUM_DEPTH 24
    #define STATUS_DISPLAY_DURATION 5000
    #define TICK_OFFSET 5

    #define QX_CONTRACT_INDEX 1

    #define EQUAL(a, b) (_mm256_movemask_epi8(_mm256_cmpeq_epi64(a, b)) == 0xFFFFFFFF)

    #define ZERO _mm256_setzero_si256()
    #define ZeroMemory(x, y) memset(x, 0, y)
    #define CopyMemory(x, y, z) memcpy(x, y, z)




/*
    void KangarooTwelve(unsigned char* input, unsigned int inputByteLen, unsigned char* output, unsigned int outputByteLen)
    {
        KangarooTwelve_F queueNode;
        KangarooTwelve_F finalNode;
        unsigned int blockNumber, queueAbsorbedLen;

        ZeroMemory(&finalNode, sizeof(KangarooTwelve_F));
        const unsigned int len = inputByteLen ^ ((K12_chunkSize ^ inputByteLen) & -(K12_chunkSize < inputByteLen));
        KangarooTwelve_F_Absorb(&finalNode, input, len);
        input += len;
        inputByteLen -= len;
        if (len == K12_chunkSize && inputByteLen)
        {
            blockNumber = 1;
            queueAbsorbedLen = 0;
            finalNode.state[finalNode.byteIOIndex] ^= 0x03;
            if (++finalNode.byteIOIndex == K12_rateInBytes)
            {
                KeccakP1600_Permute_12rounds(finalNode.state);
                finalNode.byteIOIndex = 0;
            }
            else
            {
                finalNode.byteIOIndex = (finalNode.byteIOIndex + 7) & ~7;
            }

            while (inputByteLen > 0)
            {
                const unsigned int len = K12_chunkSize ^ ((inputByteLen ^ K12_chunkSize) & -(inputByteLen < K12_chunkSize));
                ZeroMemory(&queueNode, sizeof(KangarooTwelve_F));
                KangarooTwelve_F_Absorb(&queueNode, input, len);
                input += len;
                inputByteLen -= len;
                if (len == K12_chunkSize)
                {
                    ++blockNumber;
                    queueNode.state[queueNode.byteIOIndex] ^= K12_suffixLeaf;
                    queueNode.state[K12_rateInBytes - 1] ^= 0x80;
                    KeccakP1600_Permute_12rounds(queueNode.state);
                    queueNode.byteIOIndex = K12_capacityInBytes;
                    KangarooTwelve_F_Absorb(&finalNode, queueNode.state, K12_capacityInBytes);
                }
                else
                {
                    queueAbsorbedLen = len;
                }
            }

            if (queueAbsorbedLen)
            {
                if (++queueNode.byteIOIndex == K12_rateInBytes)
                {
                    KeccakP1600_Permute_12rounds(queueNode.state);
                    queueNode.byteIOIndex = 0;
                }
                if (++queueAbsorbedLen == K12_chunkSize)
                {
                    ++blockNumber;
                    queueAbsorbedLen = 0;
                    queueNode.state[queueNode.byteIOIndex] ^= K12_suffixLeaf;
                    queueNode.state[K12_rateInBytes - 1] ^= 0x80;
                    KeccakP1600_Permute_12rounds(queueNode.state);
                    queueNode.byteIOIndex = K12_capacityInBytes;
                    KangarooTwelve_F_Absorb(&finalNode, queueNode.state, K12_capacityInBytes);
                }
            }
            else
            {
                ZeroMemory(queueNode.state, sizeof(queueNode.state));
                queueNode.byteIOIndex = 1;
                queueAbsorbedLen = 1;
            }
        }
        else
        {
            if (len == K12_chunkSize)
            {
                blockNumber = 1;
                finalNode.state[finalNode.byteIOIndex] ^= 0x03;
                if (++finalNode.byteIOIndex == K12_rateInBytes)
                {
                    KeccakP1600_Permute_12rounds(finalNode.state);
                    finalNode.byteIOIndex = 0;
                }
                else
                {
                    finalNode.byteIOIndex = (finalNode.byteIOIndex + 7) & ~7;
                }

                ZeroMemory(queueNode.state, sizeof(queueNode.state));
                queueNode.byteIOIndex = 1;
                queueAbsorbedLen = 1;
            }
            else
            {
                blockNumber = 0;
                if (++finalNode.byteIOIndex == K12_rateInBytes)
                {
                    KeccakP1600_Permute_12rounds(finalNode.state);
                    finalNode.state[0] ^= 0x07;
                }
                else
                {
                    finalNode.state[finalNode.byteIOIndex] ^= 0x07;
                }
            }
        }

        if (blockNumber)
        {
            if (queueAbsorbedLen)
            {
                blockNumber++;
                queueNode.state[queueNode.byteIOIndex] ^= K12_suffixLeaf;
                queueNode.state[K12_rateInBytes - 1] ^= 0x80;
                KeccakP1600_Permute_12rounds(queueNode.state);
                KangarooTwelve_F_Absorb(&finalNode, queueNode.state, K12_capacityInBytes);
            }
            unsigned int n = 0;
            for (unsigned long long v = --blockNumber; v && (n < sizeof(unsigned long long)); ++n, v >>= 8)
            {
            }
            unsigned char encbuf[sizeof(unsigned long long) + 1 + 2];
            for (unsigned int i = 1; i <= n; ++i)
            {
                encbuf[i - 1] = (unsigned char)(blockNumber >> (8 * (n - i)));
            }
            encbuf[n] = (unsigned char)n;
            encbuf[++n] = 0xFF;
            encbuf[++n] = 0xFF;
            KangarooTwelve_F_Absorb(&finalNode, encbuf, ++n);
            finalNode.state[finalNode.byteIOIndex] ^= 0x06;
        }
        finalNode.state[K12_rateInBytes - 1] ^= 0x80;
        KeccakP1600_Permute_12rounds(finalNode.state);
        CopyMemory(output, finalNode.state, outputByteLen);
    }
*/

/** Extendable ouput function KangarooTwelve.
    * @param  input           Pointer to the input message (M).
    * @param  inputByteLen    The length of the input message in bytes.
    * @param  output          Pointer to the output buffer.
    * @param  outputByteLen   The desired number of output bytes.
    * @param  customization   Pointer to the customization string (C).
    * @param  customByteLen   The length of the customization string in bytes.
    * @return 0 if successful, 1 otherwise.
    */
  //int KangarooTwelve(const unsigned char *input, size_t inputByteLen, unsigned char *output, size_t outputByteLen, const unsigned char *customization, size_t customByteLen);
    extern int KangarooTwelve(const unsigned char *input, size_t inputByteLen, unsigned char *output, size_t outputByteLen, const unsigned char *customization, size_t customByteLen);


    int KangarooTwelveCryptoHashFunction(const unsigned char* input, const unsigned int inputByteLen, unsigned char* output)
    {
        KangarooTwelve(input, inputByteLen, output, 64, NULL, 0);
        return 0;
    }

        /* Qubic Specific */
        typedef unsigned long long felm_t[2]; // Datatype for representing 128-bit field elements
        typedef felm_t f2elm_t[2]; // Datatype for representing quadratic extension field elements

        typedef struct
        { // Point representation in affine coordinates
            f2elm_t x;
            f2elm_t y;
        } point_affine;
        typedef point_affine point_t[1];

        extern void ecc_mul_fixed(unsigned long long* k, point_t Q);
        extern void encode(point_t P, unsigned char* Pencoded);
        extern bool decode(const unsigned char* Pencoded, point_t P);

        extern void SchnorrQ_Sign(const unsigned char* SecretKey, const unsigned char* PublicKey, const unsigned char* Message, const unsigned int SizeMessage, unsigned char* Signature);

        /* Qubic exposed Api */

        void getIdentity(unsigned char* publicKey, char* identity, bool isLowerCase)
        {
            for (int i = 0; i < 4; i++)
            {
                unsigned long long publicKeyFragment = *((unsigned long long*) & publicKey[i << 3]);
                for (int j = 0; j < 14; j++)
                {
                    identity[i * 14 + j] = publicKeyFragment % 26 + (isLowerCase ? 'a' : 'A');
                    publicKeyFragment /= 26;
                }
            }
            unsigned int identityBytesChecksum;
            KangarooTwelve(publicKey, 32, (unsigned char*)&identityBytesChecksum, 3, NULL, 0);
            identityBytesChecksum &= 0x3FFFF;
            for (int i = 0; i < 4; i++)
            {
                identity[56 + i] = identityBytesChecksum % 26 + (isLowerCase ? 'a' : 'A');
                identityBytesChecksum /= 26;
            }
            identity[60] = 0;
        }

        void getPrivateKey(unsigned char* subseed, unsigned char* privateKey)
        {
            KangarooTwelve(subseed, 32, privateKey, 32, NULL, 0);
        }


        void getPublicKey(const unsigned char* privateKey, unsigned char* publicKey)
        { // SchnorrQ public key generation
          // It produces a public key publicKey, which is the encoding of P = s*G, where G is the generator and
          // s is the output of hashing publicKey and taking the least significant 32 bytes of the result
          // Input:  32-byte privateKey
          // Output: 32-byte publicKey
            point_t P;

            ecc_mul_fixed((unsigned long long*)privateKey, P); // Compute public key
            encode(P, publicKey);                              // Encode public key
        }

            bool getPublicKeyFromIdentity(const unsigned char* identity, unsigned char* publicKey)
            {
                unsigned char publicKeyBuffer[32];
                for (int i = 0; i < 4; i++)
                {
                    *((unsigned long long*) & publicKeyBuffer[i << 3]) = 0;
                    for (int j = 14; j-- > 0; )
                    {
                        if (identity[i * 14 + j] < 'A' || identity[i * 14 + j] > 'Z')
                        {
                            return false;
                        }

                        *((unsigned long long*) & publicKeyBuffer[i << 3]) = *((unsigned long long*) & publicKeyBuffer[i << 3]) * 26 + (identity[i * 14 + j] - 'A');
                    }
                }
                unsigned int identityBytesChecksum;
                KangarooTwelve(publicKeyBuffer, 32, (unsigned char*)&identityBytesChecksum, 3, NULL, 0);
                identityBytesChecksum &= 0x3FFFF;
                for (int i = 0; i < 4; i++)
                {
                    if (identityBytesChecksum % 26 + 'A' != identity[56 + i])
                    {
                        return false;
                    }
                    identityBytesChecksum /= 26;
                }
                *((__m256i*)publicKey) = *((__m256i*)publicKeyBuffer);

                return true;
            }

            bool getSubseed(const unsigned char* seed, unsigned char* subseed)
            {
                unsigned char seedBytes[55];
                for (int i = 0; i < 55; i++)
                {
                    if (seed[i] < 'a' || seed[i] > 'z')
                    {
                        return false;
                    }
                    seedBytes[i] = seed[i] - 'a';
                }
                KangarooTwelve(seedBytes, sizeof(seedBytes), subseed, 32, NULL, 0);

                return true;
            }

            void sign(const unsigned char* subseed, const unsigned char* publicKey, const unsigned char* messageDigest, unsigned char* signature)
            {
                SchnorrQ_Sign(subseed, publicKey, messageDigest, 32, signature);
            }

}