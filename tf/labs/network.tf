resource "aws_vpc" "lab_vpc" {
  cidr_block           = var.lab_cidr
  enable_dns_hostnames = true
  tags = {
    Name = "CLAIRE Lab VPC"
  }
}

resource "aws_internet_gateway" "lab_internet_gateway" {
  vpc_id = aws_vpc.lab_vpc.id
  tags = {
    Name = "CLAIRE Lab Internet Gateway"
  }
}

resource "aws_subnet" "lab_subnet" {
  vpc_id            = aws_vpc.lab_vpc.id
  cidr_block        = var.lab_cidr
  availability_zone = var.lab_availability_zone

  tags = {
    Name = "CLAIRE Lab Public Subnet"
  }
}

resource "aws_route_table" "lab_route_table" {
  vpc_id = aws_vpc.lab_vpc.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.lab_internet_gateway.id
  }
  tags = {
    Name = "CLAIRE Lab Public Routing"
  }
}

resource "aws_route_table_association" "lab_route_assoc" {
  subnet_id      = aws_subnet.lab_subnet.id
  route_table_id = aws_route_table.lab_route_table.id
}

resource "aws_vpc_endpoint" "ssm" {
  count = var.create_vpc_endpoints

  vpc_id              = aws_vpc.lab_vpc.id
  service_name        = "com.amazonaws.${var.aws_region}.ssm"
  vpc_endpoint_type   = "Interface"
  security_group_ids  = [aws_security_group.vpc_endpoints[0].id]
  private_dns_enabled = true
}

resource "aws_vpc_endpoint_subnet_association" "ssm" {
  count = var.create_vpc_endpoints

  vpc_endpoint_id = aws_vpc_endpoint.ssm[0].id
  subnet_id       = aws_subnet.lab_subnet.id
}

resource "aws_vpc_endpoint" "ec2messages" {
  count = var.create_vpc_endpoints

  vpc_id              = aws_vpc.lab_vpc.id
  service_name        = "com.amazonaws.${var.aws_region}.ec2messages"
  vpc_endpoint_type   = "Interface"
  security_group_ids  = [aws_security_group.vpc_endpoints[0].id]
  private_dns_enabled = true
}

resource "aws_vpc_endpoint_subnet_association" "ec2messages" {
  count = var.create_vpc_endpoints

  vpc_endpoint_id = aws_vpc_endpoint.ec2messages[0].id
  subnet_id       = aws_subnet.lab_subnet.id
}

resource "aws_vpc_endpoint" "ssmmessages" {
  count = var.create_vpc_endpoints

  vpc_id              = aws_vpc.lab_vpc.id
  service_name        = "com.amazonaws.${var.aws_region}.ssmmessages"
  vpc_endpoint_type   = "Interface"
  security_group_ids  = [aws_security_group.vpc_endpoints[0].id]
  private_dns_enabled = true
}

resource "aws_vpc_endpoint_subnet_association" "ssmmessages" {
  count = var.create_vpc_endpoints

  vpc_endpoint_id = aws_vpc_endpoint.ssmmessages[0].id
  subnet_id       = aws_subnet.lab_subnet.id
}

resource "aws_security_group" "vpc_endpoints" {
  count = var.create_vpc_endpoints

  name   = "clare_lab_vpc_endpoints"
  vpc_id = aws_vpc.lab_vpc.id

  ingress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = [var.lab_cidr]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = [var.lab_cidr]
  }
}


