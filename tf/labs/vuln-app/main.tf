terraform {
  required_version = ">=0.12"
}

provider "aws" {
  version = "~>2.0"
  region  = var.aws_region
}

locals {
  ubuntu_1804_base = "ami-089cc16f7f08c4457"
}

resource "aws_instance" "vuln_app" {
  count = var.vulnerable_ami_id == local.ubuntu_1804_base ? 0 : 1

  ami                    = var.vulnerable_ami_id
  instance_type          = var.instance_type
  key_name               = var.key_name
  vpc_security_group_ids = [aws_security_group.vuln_group.id]
  tags = {
    Name        = var.vuln_app_name
    Lab         = var.vuln_app_name
    Provisioned = false
  }
}

resource "aws_instance" "vuln_app_provisioned" {
  count = var.vulnerable_ami_id == local.ubuntu_1804_base ? 1 : 0

  ami                    = var.vulnerable_ami_id
  instance_type          = var.instance_type
  key_name               = var.key_name
  vpc_security_group_ids = [aws_security_group.vuln_group.id]
  tags = {
    Name        = var.vuln_app_name
    Lab         = var.vuln_app_name
    Provisioned = true
  }

  provisioner "file" {
    source      = var.vuln_app_init_script
    destination = "/tmp/init.sh"

    connection {
      type        = "ssh"
      user        = "ubuntu"
      host        = self.public_ip
      private_key = file(var.vuln_app_pri_key)
    }
  }

  provisioner "file" {
    source      = var.vuln_app_init_sql_gz
    destination = "/tmp/init.sql.gz"

    connection {
      type        = "ssh"
      user        = "ubuntu"
      host        = self.public_ip
      private_key = file(var.vuln_app_pri_key)
    }
  }

  provisioner "remote-exec" {
    inline = [
      "chmod +x /tmp/init.sh",
      "sudo /tmp/init.sh",
      "cat /tmp/init.sql.gz | gunzip | sudo mysql ${var.vuln_app_db}",
    ]

    connection {
      type        = "ssh"
      user        = "ubuntu"
      host        = self.public_ip
      private_key = file(var.vuln_app_pri_key)
    }
  }
}

resource "aws_security_group" "vuln_group" {
  name = "vuln_security_group"

  ingress {
    protocol    = "tcp"
    from_port   = 22
    to_port     = 22
    cidr_blocks = ["${chomp(data.http.my_external_ip.body)}/32"]
  }

  ingress {
    protocol    = "tcp"
    from_port   = 80
    to_port     = 80
    cidr_blocks = ["${chomp(data.http.my_external_ip.body)}/32"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_key_pair" "vuln_app" {
  key_name   = var.key_name
  public_key = file(var.vuln_app_pub_key)
}
