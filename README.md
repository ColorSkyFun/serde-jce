# serde-jce: 一个基于 Serde 的 JCE 协议序列化/反序列化库.

这个仓库是jce协议的rust实现，使用serde.

使用示例

对于结构体
```rust
#[derive(serde::Serialize)]
struct Inner {
    #[serde(rename = "1")]
    data1: u32,
    #[serde(rename = "234", with = "serde_bytes")] // 指定为byte array形式，对应JCE的 simple list
    data2: Vec<u8>,
}

let inner = Inner {
    data1: 0xDEADBEEF,
    data2: vec![0x1, 0x2, 0x3];
};
let serialized = serde_jce::to_vec(&inner)?;
println!("{:?}", serialized);
```

由于jce的数据单元为(tag, type, value), 如果使用这样的方式序列化只能得到{ 0: {...} }的数据

```rust
let mut data = HashMap::new();
data.insert("v1", vec![12, 34]);
let serialized = crate::to_vec(&data)?;
println!("{:?}", serialized);
```

### TODO

- [x] 序列化
- [ ] 反序列化
