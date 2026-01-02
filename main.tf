terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = "us-east-1"
}

# Latest Ubuntu 22.04 AMI
data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"] # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*"]
  }
}

resource "aws_security_group" "ordinator" {
  name        = "ordinator-sg"
  description = "Allow SSH"

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_instance" "ordinator" {
  ami           = data.aws_ami.ubuntu.id
  instance_type = "t3.medium"

  security_groups = [aws_security_group.ordinator.name]

  user_data = <<-EOF
    #!/bin/bash
    set -e

    # Install Nix
    curl -L https://nixos.org/nix/install | sh -s -- --daemon

    # Enable nix-command and flakes
    mkdir -p /etc/nix
    echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

    # Install git
    apt-get update && apt-get install -y git
  EOF

  tags = {
    Name = "ordinator-build-machine"
  }
}

output "instance_ip" {
  value = aws_instance.ordinator.public_ip
}
