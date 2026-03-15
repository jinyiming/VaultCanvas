import os  
import sys  
import struct  
import argon2  
from Crypto.Cipher import AES  
from Crypto.Random import get_random_bytes  
from Crypto.Hash import SHA3_256  

# 安全配置参数  
CONFIG = {  
    "ARGON2_PARAMS": {  
        "time_cost": 2,  
        "memory_cost": 1024 * 64,  # 64MB内存占用  
        "parallelism": 2,  
        "hash_len": 32,            # AES-256密钥长度  
        "salt_len": 24             
    },  
    "AES_GCM": {  
        "iv_len": 12,              # GCM推荐IV长度  
        "mac_len": 16,             # 认证标签长度  
        "nonce_len": 16            # 防重放攻击值  
    },  
    "PASSWORD_POLICY": {  
        "min_length": 13           # 最小密码长度  
    },  
    "FILE_FORMAT": {  
        "header": b"SECURE_ENC_V5",# 文件头标识  
        "version": 5               # 版本号  
    }  
}  

# 初始化Argon2  
ph = argon2.PasswordHasher(  
    time_cost=CONFIG["ARGON2_PARAMS"]["time_cost"],  
    memory_cost=CONFIG["ARGON2_PARAMS"]["memory_cost"],  
    parallelism=CONFIG["ARGON2_PARAMS"]["parallelism"],  
    hash_len=CONFIG["ARGON2_PARAMS"]["hash_len"],  
    salt_len=CONFIG["ARGON2_PARAMS"]["salt_len"],  
    type=argon2.Type.ID  
)  

class SecureMemory:  
    """敏感数据内存安全处理"""  
    @staticmethod  
    def wipe(data: bytes):  
        if isinstance(data, bytes):  
            data_bytes = bytearray(data)  
            for i in range(len(data_bytes)):  
                data_bytes[i] = 0x00  
            del data_bytes  

def validate_password(password: str):  
    """密码长度验证"""  
    if len(password) < CONFIG["PASSWORD_POLICY"]["min_length"]:  
        raise ValueError(f"密码至少需要{CONFIG['PASSWORD_POLICY']['min_length']}个字符")  

def generate_key(password: str, salt: bytes) -> bytes:  
    """Argon2id密钥派生"""  
    try:  
        return argon2.low_level.hash_secret_raw(  
            secret=password.encode('utf-8'),  
            salt=salt,  
            time_cost=CONFIG["ARGON2_PARAMS"]["time_cost"],  
            memory_cost=CONFIG["ARGON2_PARAMS"]["memory_cost"],  
            parallelism=CONFIG["ARGON2_PARAMS"]["parallelism"],  
            hash_len=CONFIG["ARGON2_PARAMS"]["hash_len"],  
            type=argon2.Type.ID  
        )  
    except Exception as e:  
        raise ValueError("密钥生成失败") from e  
    finally:  
        SecureMemory.wipe(password.encode('utf-8'))  

def encrypt_file(file_path: str, main_password: str, id_password: str):  
    """加密流程（带元数据完整性验证）"""  
    validate_password(main_password)  
    
    try:  
        # 生成加密要素  
        salt = get_random_bytes(CONFIG["ARGON2_PARAMS"]["salt_len"])  
        iv = get_random_bytes(CONFIG["AES_GCM"]["iv_len"])  
        nonce = get_random_bytes(CONFIG["AES_GCM"]["nonce_len"])  
        
        # 生成密钥  
        key = generate_key(main_password, salt)  
        cipher = AES.new(key, AES.MODE_GCM, nonce=iv, mac_len=CONFIG["AES_GCM"]["mac_len"])  
        cipher.update(nonce)  # 绑定防重放随机数  
        
        # 读取并加密数据  
        with open(file_path, 'rb') as f:  
            plaintext = f.read()  
        
        ciphertext, tag = cipher.encrypt_and_digest(plaintext)  
        
        # 生成元数据签名  
        metadata = salt + iv + nonce + tag  
        id_hash = SHA3_256.new(id_password.encode()).digest()  
        metadata_sign = SHA3_256.new(metadata + id_hash).digest()  
        
        # 写入加密文件  
        enc_file_path = file_path + '.enc'  
        with open(enc_file_path, 'wb') as f:  
            f.write(CONFIG["FILE_FORMAT"]["header"])  
            f.write(struct.pack('B', CONFIG["FILE_FORMAT"]["version"]))  
            f.write(metadata)  
            f.write(metadata_sign)  
            f.write(id_hash)  
            f.write(ciphertext)  
        
        print(f"[成功] 文件已加密: {enc_file_path}")  
        return enc_file_path  
    except Exception as e:  
        print(f"[错误] 加密失败: {str(e)}")  
        if 'enc_file_path' in locals() and os.path.exists(enc_file_path):  
            os.remove(enc_file_path)  
        sys.exit(1)  
    finally:  
        if 'key' in locals():  
            SecureMemory.wipe(key)  

def decrypt_file(enc_file_path: str, main_password: str, id_password: str):  
    """解密流程（带完整安全验证）"""  
    dec_file_path = None  
    try:  
        # 验证文件存在性  
        if not os.path.isfile(enc_file_path):  
            raise FileNotFoundError("加密文件不存在")  
        
        # 读取加密文件  
        with open(enc_file_path, 'rb') as f:  
            header = f.read(len(CONFIG["FILE_FORMAT"]["header"]))  
            if header != CONFIG["FILE_FORMAT"]["header"]:  
                raise ValueError("无效加密文件格式")  
            
            version = struct.unpack('B', f.read(1))[0]  
            if version != CONFIG["FILE_FORMAT"]["version"]:  
                raise ValueError("不兼容的版本号")  
            
            # 提取元数据  
            salt = f.read(CONFIG["ARGON2_PARAMS"]["salt_len"])  
            iv = f.read(CONFIG["AES_GCM"]["iv_len"])  
            nonce = f.read(CONFIG["AES_GCM"]["nonce_len"])  
            tag = f.read(CONFIG["AES_GCM"]["mac_len"])  
            metadata = salt + iv + nonce + tag  
            
            metadata_sign = f.read(SHA3_256.digest_size)  
            id_hash = f.read(SHA3_256.digest_size)  
            ciphertext = f.read()  
        
        # 验证ID密码  
        expected_id_hash = SHA3_256.new(id_password.encode()).digest()  
        if id_hash != expected_id_hash:  
            raise ValueError("ID密码验证失败")  
        
        # 验证元数据完整性  
        computed_sign = SHA3_256.new(metadata + id_hash).digest()  
        if computed_sign != metadata_sign:  
            raise ValueError("文件完整性校验失败")  
        
        # 生成密钥  
        key = generate_key(main_password, salt)  
        cipher = AES.new(key, AES.MODE_GCM, nonce=iv, mac_len=CONFIG["AES_GCM"]["mac_len"])  
        cipher.update(nonce)  
        
        # 解密数据  
        plaintext = cipher.decrypt_and_verify(ciphertext, tag)  
        
        # 生成解密文件名  
        if enc_file_path.endswith('.enc'):  
            dec_file_path = enc_file_path[:-4] + '.dec'  
        else:  
            dec_file_path = enc_file_path + '.dec'  
        
        # 写入解密文件  
        with open(dec_file_path, 'wb') as f:  
            f.write(plaintext)  
        
        print(f"[成功] 文件已解密: {dec_file_path}")  
        return dec_file_path  
    except Exception as e:  
        print(f"[错误] 解密失败: {str(e)}")  
        if dec_file_path and os.path.exists(dec_file_path):  
            os.remove(dec_file_path)  
        sys.exit(1)  
    finally:  
        if 'key' in locals():  
            SecureMemory.wipe(key)  
        if 'plaintext' in locals():  
            SecureMemory.wipe(plaintext)  

if __name__ == "__main__":  
    try:  
        # 用户交互  
        operation = input("请选择操作 (encrypt/decrypt): ").lower().strip()  
        file_path = input("文件路径: ").strip()  
        
        # 基础验证  
        if not file_path:  
            raise ValueError("文件路径不能为空")  
        
        # 操作分发  
        if operation == "encrypt":  
            if not os.path.isfile(file_path):  
                raise FileNotFoundError("原始文件不存在")  
            main_password = input("主密码: ").strip()  
            id_password = input("ID密码: ").strip()  
            encrypt_file(file_path, main_password, id_password)  
            
        elif operation == "decrypt":  
            if not os.path.isfile(file_path):  
                raise FileNotFoundError("加密文件不存在")  
            if not file_path.endswith('.enc'):  
                print("[提示] 通常加密文件以 .enc 结尾")  
            main_password = input("主密码: ").strip()  
            id_password = input("ID密码: ").strip()  
            decrypt_file(file_path, main_password, id_password)  
            
        else:  
            raise ValueError("无效操作类型")  
            
    except KeyboardInterrupt:  
        print("\n操作已取消")  
        sys.exit(0)  
    except Exception as e:  
        print(f"[严重错误] {str(e)}")  
        sys.exit(1)  