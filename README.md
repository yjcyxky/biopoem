## Biopoem

### 前提条件

- 安装[`terraform`](https://learn.hashicorp.com/tutorials/terraform/install-cli)
- 新创建一个工作目录，例如`biopoem-instance`
- 准备模板文件template.tf，将其保存至`biopoem-instance/templates`目录
  ```
    resource "alicloud_vpc" "vpc" {
      name       = "biopoem-vpc"
      cidr_block = "172.16.0.0/12"
    }

    resource "alicloud_vswitch" "vsw" {
      vpc_id            = alicloud_vpc.vpc.id
      cidr_block        = "172.16.0.0/21"
      zone_id           = "{{ zone }}"
    }

    resource "alicloud_security_group" "default" {
      name = "biopoem-security_group"
      vpc_id = alicloud_vpc.vpc.id
    }

    resource "alicloud_security_group_rule" "allow_all_tcp" {
      type              = "ingress"
      ip_protocol       = "tcp"
      nic_type          = "intranet"
      policy            = "accept"
      port_range        = "1/65535"
      priority          = 1
      security_group_id = alicloud_security_group.default.id
      cidr_ip           = "0.0.0.0/0"
    }

    module "tf-instances" {  
      source                      = "alibaba/ecs-instance/alicloud"  
      region                      = "{{ region }}"  
      number_of_instances         = "{{ num_of_hosts }}"  
      vswitch_id                  = alicloud_vswitch.vsw.id  
      group_ids                   = [alicloud_security_group.default.id]  
      private_ips                 = [{% for ipaddr in ipaddrs %}"{{ ipaddr }}", {% endfor %}]
      image_ids                   = ["{{ image }}"]  
      instance_type               = "{{ instance_type }}" 
      key_name                    = "{{ keypair_name }}"
      internet_max_bandwidth_out  = 100
      internet_max_bandwidth_in   = 100
      associate_public_ip_address = false  
      instance_name               = "biopoem_instance"  
      host_name                   = "biopoem"  
      internet_charge_type        = "PayByTraffic"   
      system_disk_category        = "cloud_essd"
    }

    output "public_ips" {
      value = "${module.tf-instances.this_public_ip}"
    }
  ```
- 准备任务模板文件`dag.template`，以调用wget下载文件为例。其中`{{ filelink }}`是模板变量，其值由传入的variables文件定义

  ```
  {
    "schema": "iglu:com.snowplowanalytics.factotum/factfile/jsonschema/1-0-0",
    "data": {
        "name": "Biopoem Testing",
        "tasks": [
            {
                "name": "Download file",
                "executor": "shell",
                "command": "wget",
                "arguments": [ "--no-check-certificate", "{{filelink}}" ],
                "dependsOn": [],
                "onResult": {
                    "terminateJobWithSuccess": [],
                    "continueJob": [ 0 ]
                }
            }
        ]
    }
  }
  ```
- 准备`variables`文件（如下示例），variables文件中的键是调度机器的主机名（当前支持同一时间最多调度255台机器，从`biopoem001`-`biopoem255`），值是一个字典，其中包括前述任务模板所引用的变量的值。
  ```
  {
    "biopoem001": {
      "filelink": "https://www.biosino.org/download/node/data/public/OED006624"
    },
    "biopoem002": {
      "filelink": "https://www.biosino.org/download/node/data/public/OED006625"
    },
    "biopoem003": {
      "filelink": "https://www.biosino.org/download/node/data/public/OED006626"
    }
  }
  ```

### `biopoem`帮助文档

由三个子命令组成，`deployer`、`server`与`client`:
- `deployer`命令，在用户端电脑上运行。用于在阿里云上部署指定机型的若干数目机器（最大支持255台）
- `server`命令，在用户端电脑上运行。用于连接部署的机器，并将DAG任务发送至每台机器，启动运算
- `client`命令，在阿里云服务器上运行，由其监控DAG任务状态，并提供远程查询接口

```
Biopoem for DAG Task with Large-scale Servers. 0.1.0
Jingcheng Yang <yjcyxky@163.com>
A suite of programs for handling big omics data

USAGE:
    biopoem [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      A flag which control whether show more messages, true if used in the command line
    -V, --version    Prints version information
    -v, --verbose    The number of occurrences of the `v/verbose` flag Verbose mode (-v, -vv, -vvv, etc.)

SUBCOMMANDS:
    client      Client for Biopoem
    deployer    Deployer for Biopoem
    help        Prints this message or the help of the given subcommand(s)
    server      Server for Biopoem
```