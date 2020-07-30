terraform {
  required_version = ">=0.12"
}

provider "aws" {
  version = "~>2.0"
  region  = var.aws_region
}

resource "aws_instance" "vuln_app" {
  ami                    = var.vulnerable_ami_id
  instance_type          = var.instance_type
  key_name               = var.key_name
  vpc_security_group_ids = [aws_security_group.vuln_group.id]
  iam_instance_profile   = aws_iam_instance_profile.lab_profile.name
  tags = {
    Name = "Vulnerable Drupal"
    Lab  = "vuln_app"
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

resource "aws_iam_instance_profile" "lab_profile" {
  name = "lab_profile"
  role = aws_iam_role.lab_role.name
}

resource "aws_iam_role" "lab_role" {
  name               = "lab_role"
  assume_role_policy = data.aws_iam_policy_document.lab_role.json
}

data "aws_iam_policy_document" "lab_role" {
  statement {
    actions = [
      "sts:AssumeRole",
    ]

    principals {
      type = "Service"
      identifiers = [
        "ec2.amazonaws.com",
      ]
    }
  }
}

data "aws_iam_policy" "AmazonEc2RoleForSSM" {
  arn = "arn:aws:iam::aws:policy/service-role/AmazonEC2RoleforSSM"
}

resource "aws_iam_role_policy_attachment" "evidence_extractor_ssm_access" {
  role       = aws_iam_role.lab_role.id
  policy_arn = data.aws_iam_policy.AmazonEc2RoleForSSM.arn
}
