# Simple redis server

## 作业
### 为 simple-redis 实现你想实现的命令，比如：
- echo command:  https://redis.io/commands/echo/
- hmget command:  https://redis.io/commands/hmget/
- sadd/sismember  https://redis.io/commands/sismember/

#### 实现echo/hmget/sadd/sismember/smembers
#### 测试：
```shell
127.0.0.1:6379> echo "hello world"
"hello world"
127.0.0.1:6379> hset mykey field1 value1
OK
127.0.0.1:6379> hset mykey field2 value2
OK
127.0.0.1:6379> hmget mykey field1 field2 field3
1) "value1"
2) "value2"
3) (nil)
127.0.0.1:6379> sadd mykey "hello" "world"
(integer) 2
127.0.0.1:6379> sadd mykey "hello"
(integer) 0
127.0.0.1:6379> sismember mykey "hello"
(integer) 1
127.0.0.1:6379> sismember mykey "hi"
(integer) 0
127.0.0.1:6379> smembers mykey
1) "world"
2) "hello"
```

### 重构代码：
- 删除 NullBulkString / NullArray
- 重构 BulkString / RespArray 代码，使其直接处理上面两种情况

#### 通过Option\<T\> 来处理两种情况，使得代码更简洁
```rust
pub struct RespArray(pub(crate) Option<Vec<RespFrame>>);
pub struct BulkString(pub(crate) Option<Vec<u8>>);
```
