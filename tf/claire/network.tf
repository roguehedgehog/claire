resource "aws_vpc" "claire_vpc" {
  cidr_block = var.claire_cidr
  tags = {
    Name = "CLAIRE Quaratine VPC"
  }
}

resource "aws_internet_gateway" "claire_internet_gateway" {
  vpc_id = aws_vpc.claire_vpc.id
  tags = {
    Name = "CLAIRE Internet Gateway"
  }
}

resource "aws_subnet" "claire_public_subnet" {
  vpc_id            = aws_vpc.claire_vpc.id
  cidr_block        = var.claire_public_cidr
  availability_zone = var.claire_az
  tags = {
    Name = "CLAIRE Public Subnet"
  }
}

resource "aws_route_table" "claire_route_table" {
  vpc_id = aws_vpc.claire_vpc.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.claire_internet_gateway.id
  }
  tags = {
    Name = "CLAIRE Public Routing"
  }
}

resource "aws_route_table_association" "claire_route_assoc" {
  subnet_id      = aws_subnet.claire_public_subnet.id
  route_table_id = aws_route_table.claire_route_table.id
}
