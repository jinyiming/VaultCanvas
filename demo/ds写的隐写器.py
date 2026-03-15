import os  
import zlib  
import struct  
import hashlib  
from Crypto.Cipher import AES  
from Crypto.Util.Padding import pad, unpad  
from Crypto.Protocol.KDF import PBKDF2  

def embed_advanced(image_path, file_path, output_path, password):  
    """安全文件嵌入（完整版）"""  
    try:  
        # 读取原始图片  
        with open(image_path, 'rb') as f:  
            img_data = f.read()  

        # 读取并处理隐藏文件  
        with open(file_path, 'rb') as f:  
            file_data = f.read()  
        
        # 数据压缩处理  
        compressed = zlib.compress(file_data, level=9)  
        
        # 生成加密要素  
        salt = os.urandom(16)  
        iv = os.urandom(16)  
        key = PBKDF2(password, salt, dkLen=32, count=100000)  
        cipher = AES.new(key, AES.MODE_CBC, iv=iv)  
        
        # 加密数据  
        encrypted = cipher.encrypt(pad(compressed, AES.block_size))  
        
        # 构建隐藏数据包  
        hidden_payload = (  
            b'\x89STE'  # 4字节伪头  
            + struct.pack('>I', len(encrypted))  # 4字节数据长度  
            + iv  # 16字节  
            + salt  # 16字节  
            + encrypted  # 可变长度加密数据  
            + hashlib.sha256(encrypted).digest()  # 32字节校验  
        )  

        # 写入合成文件  
        with open(output_path, 'wb') as f:  
            f.write(img_data)  
            f.write(hidden_payload)  
            f.flush()  
            if hasattr(os, 'fdatasync'):  
                os.fdatasync(f.fileno())  
            else:  
                os.fsync(f.fileno())  

        print(f"安全嵌入完成 -> {output_path}")  
        print(f"密钥指纹: {hashlib.sha256(key).hexdigest()[:12]}")  

    except Exception as e:  
        print(f"嵌入失败: {str(e)}")  

def extract_advanced(image_path, output_path, password):  
    """安全文件提取（完整版）"""  
    try:  
        with open(image_path, 'rb') as f:  
            data = f.read()  

        # 基本文件校验  
        if len(data) < 100:  
            raise ValueError("文件尺寸异常")  

        # 定位隐藏数据  
        start_marker = b'\x89STE'  
        start = data.rfind(start_marker)  
        if start == -1:  
            raise ValueError("未检测到有效载荷")  

        # 解析包头结构  
        header_start = start + 4  
        header = data[header_start : header_start + 36]  
        data_len, iv, salt = struct.unpack('>I16s16s', header)  

        # 计算数据区段  
        encrypted_start = header_start + 36  
        encrypted_end = encrypted_start + data_len  
        encrypted = data[encrypted_start : encrypted_end]  
        checksum = data[encrypted_end : encrypted_end + 32]  

        # 完整性验证  
        if hashlib.sha256(encrypted).digest() != checksum:  
            raise ValueError("数据校验失败")  

        # 密钥派生与解密  
        key = PBKDF2(password, salt, dkLen=32, count=100000)  
        cipher = AES.new(key, AES.MODE_CBC, iv=iv)  
        decrypted = unpad(cipher.decrypt(encrypted), AES.block_size)  
        original = zlib.decompress(decrypted)  

        # 写入输出文件  
        with open(output_path, 'wb') as f:  
            f.write(original)  

        print(f"安全提取至: {output_path}")  
        return True  

    except Exception as e:  
        print(f"提取失败: {str(e)}")  
        return False  

if __name__ == "__main__":  
    mode = input("选择模式 [1]嵌入 [2]提取: ")  
    pwd = input("输入密码: ").encode('utf-8')  

    try:  
        if mode == '1':  
            embed_advanced(  
                input("原始图片路径: "),  
                input("隐藏文件路径: "),  
                input("输出路径: "),  
                pwd  
            )  
        elif mode == '2':  
            extract_advanced(  
                input("隐写图片路径: "),  
                input("提取路径: "),  
                pwd  
            )  
        else:  
            print("无效模式选择")  
    except KeyboardInterrupt:  
        print("\n操作已取消") 