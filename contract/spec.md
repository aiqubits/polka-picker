## pickers 合约

### Picker授权支付智能合约设计

内部业务实现: 确认支付金额，支付price，内部合约进行转账，校验picker结构中的picker_id, dev_user_id, dev_wallet_address, 与传入参数是否一致，如果一致把转移过来的金额的 95% 转给dev_wallet_address。剩余 5% 存在合约之中，实现一个函数，只有合约发行者才可以提取合约中的剩余资金。

#### 接口设计

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import "@openzeppelin/contracts/utils/Address.sol";

using EnumerableSet for EnumerableSet.AddressSet;
using Address for address payable;

// 角色定义
bytes32 public constant OPERATOR_ROLE = keccak256("OPERATOR_ROLE");

// 资金分配比例
uint256 public constant DEV_SHARE_PERCENT = 95;
uint256 public constant FEE_PERCENT = 5;

picker数据存储: picker_id, dev_user_id, dev_wallet_address

struct Picker {
    bytes32 pickerId;
    uint256 devUserId;
    address devWalletAddress;
}

mapping(bytes32 => Picker) public pickers; // pickerId => Picker
mapping(address => bytes32) public walletToPickerId; // 快速根据钱包查PickerId

admin数据存储: admin_address

EnumerableSet.AddressSet private adminAddresses; // 管理员地址集合（需迭代）

#### 事件日志

event PickerRegistered(bytes32 indexed pickerId, address indexed wallet);
event PickerRemoved(bytes32 indexed pickerId);
event PaymentProcessed(bytes32 indexed pickerId, uint256 amount);
event FundsWithdrawn(address indexed admin, uint256 amount);

// 初始化合约并设置管理员
constructor(address defaultAdmin) {
    _grantRole(DEFAULT_ADMIN_ROLE, defaultAdmin);
    adminAddresses.add(defaultAdmin);
}

##### 函数实现

实现一个 grantOperatorRole 授权地址为操作员的函数，只有发行者可以授权某个钱包地址为操作员。

实现一个 revokeOperatorRole 取消操作员权限的函数，只有合约发行者可以取消授权某个钱包地址为操作员。

实现一个 registerPicker 函数，只有被授权的钱包地址拥有特定权限，该权限是可以在 Picker 数据结构中添加新的 pickerId, devUserId, devWalletAddress

function registerPicker(
    bytes32 pickerId,
    uint256 devUserId,
    address devWalletAddress
) external onlyRole(OPERATOR_ROLE);

实现一个 removePicker 函数，只有合约发行者可以删除 Picker 数据结构中的 pickerId, devUserId, devWalletAddress

function removePicker(bytes32 pickerId) external onlyRole(DEFAULT_ADMIN_ROLE);

实现一个 pay 函支付函数，校验 Picker 信息并分配资金，按照比例 95% 直接转移给开发者的钱包地址，按照比例剩余接近 5% 存储在合约中

function pay(
    bytes32 pickerId,
    uint256 devUserId,
    address devWalletAddress
) external payable;

实现一个 withdrawFunds 提取合约余额的函数，仅 DEFAULT_ADMIN_ROLE 可调用

function withdrawFunds(address payable recipient) external onlyRole(DEFAULT_ADMIN_ROLE);

实现一个 queryPickerByWallet 查询 Picker 信息的函数，所有人都可以根据传入的钱包地址查询 Picker 数据结构中是否包含该地址的 pickerId, devUserId ，如果包含则返回(pickerId, devUserId)，否则返回()

function queryPickerByWallet(address wallet) external view returns (bytes32, uint256);

实现一个 getAllPickers 查询 PickerID 列表函数，只有发行者可以查询 Picker 数据结构中的 pickerId, devUserId, devWalletAddress 列表

function getAllPickers() external view returns (Picker[] memory);

实现一个 getAllAdmins 查询所有管理员地址列表的函数，只有发行者可以查询 adminAddresses 中的 address 列表

function getAllAdmins() external view returns (address[] memory);

实现一个 isAdmin // 检查某个钱包地址是否为管理员的函数，所有人都可以查询，传入一个钱包地址，如果包含在 adminAddresses 中则返回 true 否则返回 false

function isAdmin(address account) external view returns (bool);




等待后端server 进行链上订单交易验证，验证成功后在后端继续完成订单。



## 示例合约

### token 工厂合约