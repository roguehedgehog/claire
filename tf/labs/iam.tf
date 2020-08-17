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

data "aws_iam_policy" "AmazonSSMManagedInstanceCore" {
  arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

resource "aws_iam_role_policy_attachment" "lab_ssm_access" {
  role       = aws_iam_role.lab_role.id
  policy_arn = data.aws_iam_policy.AmazonSSMManagedInstanceCore.arn
}

data "aws_iam_policy_document" "allow_all_s3_access" {
  statement {
    effect = "Allow"
    actions = [
      "s3:*"
    ]
    resources = [
      "${aws_s3_bucket.target_bucket.arn}/*"
    ]
  }
}

resource "aws_iam_policy" "allow_all_s3_access_policy" {
  name   = "claire_lab_allow_s3_access_policy"
  policy = data.aws_iam_policy_document.allow_all_s3_access.json
}

resource "aws_iam_role_policy_attachment" "lab_s3_access" {
  role       = aws_iam_role.lab_role.id
  policy_arn = aws_iam_policy.allow_all_s3_access_policy.arn
}
